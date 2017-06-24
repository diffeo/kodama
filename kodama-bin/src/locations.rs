extern crate byteorder;
#[macro_use]
extern crate clap;
extern crate csv;
extern crate kodama;
extern crate rayon;
extern crate serde;
#[macro_use]
extern crate serde_derive;

use std::fs::File;
use std::io::{self, Write};
use std::path::Path;
use std::process;
use std::time::Instant;

use byteorder::{ByteOrder, LittleEndian};
use kodama::Method;
use rayon::prelude::*;

/// The type of a CSV record in our location data.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Location {
    city: String,
    region: String,
    country: String,
    latitude: f64,
    longitude: f64,
}

/// The type of a CSV record in our output data (a step in the dendrogram).
#[derive(Debug, Serialize)]
struct Step {
    cluster1: usize,
    cluster2: usize,
    dissimilarity: f64,
    size: usize,
}

/// Parse CSV location data from any reader.
fn parse_csv<R: io::Read>(rdr: R) -> io::Result<Vec<Location>> {
    let mut locations = vec![];
    let mut csvrdr = csv::Reader::from_reader(rdr);
    for result in csvrdr.deserialize() {
        let location = result?;
        locations.push(location);
    }
    Ok(locations)
}

/// Compute the distance between a pair of location records.
///
/// The coordinates of each location are given by signed decimal degrees of
/// latitude/longitude. Positive degrees corresponds to North/East while
/// negative degrees corresponds to South/West.
///
/// Note that the distance returned is "as the crow flies."
///
/// See: https://en.wikipedia.org/wiki/Haversine_formula
fn haversine(loc1: &Location, loc2: &Location) -> f64 {
    const EARTH_RADIUS: f64 = 3958.756; // miles

    let (lat1, lon1, lat2, lon2) = (
        loc1.latitude.to_radians(),
        loc1.longitude.to_radians(),
        loc2.latitude.to_radians(),
        loc2.longitude.to_radians(),
    );
    let delta_lat = lat2 - lat1;
    let delta_lon = lon2 - lon1;
    let a =
        (delta_lat / 2.0).sin().powi(2)
        + lat1.cos() * lat2.cos() * (delta_lon / 2.0).sin().powi(2);
    2.0 * EARTH_RADIUS * a.sqrt().atan()
}

/// Return a condensed pairwise distance matrix for all pairs of locations
/// given.
///
/// The distance between each pair is computed by the given `distance`
/// function.
fn condensed_distance_matrix<F>(
    records: &[Location],
    distance: F,
) -> Vec<f64>
where F: Fn(&Location, &Location) -> f64 + Send + Sync + 'static
{
    // We write this "functionally" so that we can benefit from easy
    // data parallelism.
    (0..records.len())
        .into_par_iter()
        // Iterate over (0, 1), (0, 2), ..., (1, 2), (1, 3), ..., (n-1, n)
        .flat_map(|i| {
            ((i+1)..records.len()).into_par_iter().map(move |j| (i, j))
        })
        .map(|(i, j)| distance(&records[i], &records[j]))
        .collect()
}

/// Run the program using the results of argv parsing.
///
/// If there was a problem, return an error.
fn run(matches: clap::ArgMatches) -> io::Result<()> {
    // Get the user supplied method. Default to single linkage.
    let method: Method = matches
        .value_of("method")
        .map(|s| s.parse())
        .unwrap_or(Ok(Method::Single))?;
    // Parse the location CSV data.
    let location_path = matches.value_of_os("location-data").unwrap();
    let locations = parse_csv(File::open(location_path)?)?;

    // Either compute the distance matrix or load it from a file.
    let start = Instant::now();
    let mut condensed = match matches.value_of_os("load-dist-from") {
        None => condensed_distance_matrix(&locations, haversine),
        Some(path) => vec_f64_from_file::<LittleEndian>(Path::new(path))?,
    };
    eprintln!("load condensed matrix took: {:?}", start.elapsed());

    // Save the distance matrix to a file if requested.
    if let Some(path) = matches.value_of_os("save-dist-to") {
        let start = Instant::now();
        vec_f64_to_file::<LittleEndian>(Path::new(path), &condensed)?;
        eprintln!("writing matrix took: {:?}", start.elapsed());
    }

    // Run linkage clustering.
    let start = Instant::now();
    let dendrogram = kodama::linkage(&mut condensed, locations.len(), method);
    eprintln!("linkage took: {:?}", start.elapsed());

    // Write the dendrogram steps to stdout in CSV format.
    let mut csvwtr = csv::Writer::from_writer(io::stdout());
    for step in dendrogram.steps() {
        csvwtr.serialize(Step {
            cluster1: step.cluster1,
            cluster2: step.cluster2,
            dissimilarity: step.dissimilarity,
            size: step.size,
        })?;
    }
    csvwtr.flush()?;
    Ok(())
}

fn main() {
    use clap::{App, AppSettings, Arg};

    // Set up the argv parser.
    let app = App::new("locations")
        .author(crate_authors!())
        .version(crate_version!())
        .max_term_width(100)
        .setting(AppSettings::UnifiedHelpMessage)
        .arg(Arg::with_name("location-data")
            .required(true)
            .help("CSV with the following columns: \
                   City,Region,Country,Latitude,Longitude. \
                   Latitude and Longitude should be in degrees."))
        .arg(Arg::with_name("load-dist-from")
            .long("load-dist-from")
            .takes_value(true))
        .arg(Arg::with_name("save-dist-to")
            .long("save-dist-to")
            .takes_value(true))
        .arg(Arg::with_name("method")
            .long("method")
            .takes_value(true));

    // Run the program. If there was an error, print it.
    if let Err(err) = run(app.get_matches()) {
        writeln!(&mut io::stderr(), "{}", err).unwrap();
        process::exit(1);
    }
}

/// Read a contiguous sequence of double-precision floating point numbers
/// from a file. The type parameter `T` indicates the endianness.
fn vec_f64_from_file<T: ByteOrder>(fpath: &Path) -> io::Result<Vec<f64>> {
    use byteorder::ReadBytesExt;

    let mut file = File::open(fpath)?;
    let len = file.metadata()?.len() as usize;
    if len % 8 != 0 {
        let msg = format!("len must be multiple of 8, found {}", len);
        return Err(io::Error::new(io::ErrorKind::Other, msg));
    }

    let mut numbers = vec![0.0; len / 8];
    unsafe {
        file.read_f64_into_unchecked::<T>(&mut numbers)?;
    }
    Ok(numbers)
}

/// Write a contiguous sequence of double-precision floating point numbers
/// to a file. The parameter `T` indicates the endianness.
fn vec_f64_to_file<T: ByteOrder>(
    fpath: &Path,
    data: &[f64],
) -> io::Result<()> {
    let mut bytes = vec![0; data.len() * 8];
    T::write_f64_into(data, &mut bytes);
    File::create(fpath)?.write_all(&bytes)
}
