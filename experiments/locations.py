'''
This program provides a command line interface to run hierarchical
agglomerative clustering on location data. Location data is a sequence
of geographical regions, where each region is represented by a
single latitude/longitude point. Clustering is then performed by
grouping closer locations together, based on the distance between each
location's latitude/longitude.

The final result of this program is a dendrogram[1]. A dendrogram is a
tree diagram that illustrates the hierarchical nature of the clusters
that were formed. The dendrogram is represented by a sequence of steps
required to build the dendrogram, and could be used as input to another
process. However, this tool also comes with a --plot flag, which will
render the dendrogram into a visual display in a GUI window.

[1] - https://en.wikipedia.org/wiki/Dendrogram
'''
from __future__ import absolute_import, division, print_function

import argparse
import csv
import math
import sys
import time

from fastcluster import linkage as linkage_fast
import matplotlib.pyplot as plt
import numpy as np
from scipy.cluster.hierarchy import dendrogram, linkage as linkage_sci


CSV_RECORD_NAMES = ['City', 'Region', 'Country', 'Latitude', 'Longitude']
CSV_DENDROGRAM_NAMES = ['cluster1', 'cluster2', 'dissimilarity', 'size']


def eprint(*args, **kwargs):
    'Like print(), but for stderr'
    kwargs['file'] = sys.stderr
    print(*args, **kwargs)


def parse_csv(rdr):
    '''
    Parses CSV from the file-like `rdr` object.

    Returns a list of records, where each record is a dict with
    keys corresponding to `CSV_RECORD_NAMES`.
    '''
    records = []
    for row in csv.DictReader(rdr):
        row['Country'] = row['Country'].upper()
        row['Latitude'] = float(row['Latitude'])
        row['Longitude'] = float(row['Longitude'])
        records.append(row)
    return records


def haversine(record1, record2):
    '''
    Compute the distance between a pair of records.

    The coordinates are given by signed decimal degrees of
    latitude/longitude. Positive degrees corresponds to North/East
    while negative degrees corresponds to South/West. Coordinates
    should be in the `Latitude` and `Longitude` keys of each record.

    Note that this distance is "as the crow flies."

    See: https://en.wikipedia.org/wiki/Haversine_formula
    '''
    def degree_to_radians(degree):
        return degree * (math.pi / 180.0)

    EARTH_RADIUS = 3958.756  # miles
    lat1, lon1 = record1['Latitude'], record1['Longitude']
    lat2, lon2 = record2['Latitude'], record2['Longitude']
    lat1, lon1, lat2, lon2 = map(degree_to_radians, [lat1, lon1, lat2, lon2])

    delta_lat = lat2 - lat1
    delta_lon = lon2 - lon1
    a = ((math.sin(delta_lat / 2) ** 2)
         + (math.cos(lat1) * math.cos(lat2) * (math.sin(delta_lon / 2) ** 2)))
    return 2 * EARTH_RADIUS * math.atan(math.sqrt(a))


def condensed_dissimilarity_matrix(records, dist=haversine):
    '''
    Computes a pairwise dissimilarity matrix for `records`.

    Specifically, a dissimilarity is computed between each pair of
    locations in `records`. The matrix returned is condensed to the
    upper triangle. A coordinate pair can be translated to an index
    into this condensed representation using `matrix_to_condensed_idx`.

    The dissimilarity between each pair of records is computed
    by `dist`. The `dist` function is given two arguments, each
    corresponding to a record.
    '''
    size = len(records)
    condensed = np.zeros((size * (size - 1)) // 2, dtype=np.float64)
    for i in range(size):
        for j in range(i+1, size):
            d = dist(records[i], records[j])
            condensed[matrix_to_condensed_idx(size, i, j)] = d
    return condensed


def matrix_to_condensed_idx(size, i, j):
    '''
    Convert a matrix index to a condensed index.

    Convert a matrix index `i,j` with size `size` to a condensed index
    into a flattened array of the matrix's upper triangle.
    '''
    assert 0 <= i
    assert i < j
    assert j < size
    return (size * i) - ((i * (i + 1)) // 2) - 1 + j - i


def condensed_matrix_from_args(args, records):
    '''
    Builds a condensed dissimilary matrix.

    `args` should be the result of parsing CLI arguments. The CLI
    arguments may indicate that the matrix can be loaded from a
    pre-computed file. Similarly, the CLI arguments may indicate
    that the matrix should be saved to a file.

    `records` should be the records parsed from the source data.
    '''
    condensed = None
    if args.load_dist_from is not None:
        start = time.time()
        with open(args.load_dist_from) as rdr:
            condensed = np.fromfile(rdr, dtype=np.float64)
        eprint('load condensed matrix took: %fs' % (time.time() - start))
    if condensed is None:
        start = time.time()
        condensed = condensed_dissimilarity_matrix(records)
        eprint('building condensed matrix took: %fs' % (time.time() - start))
    if args.save_dist_to is not None:
        start = time.time()
        with open(args.save_dist_to, 'w+') as wtr:
            condensed.tofile(wtr)
        eprint('writing condensed matrix took: %fs' % (time.time() - start))
    return condensed


def plot(records, dendrogram_steps):
    'Black magic to plot a dendrogram.'
    plt.figure()
    labels = [r['City'] for r in records]
    data = dendrogram(steps, labels=labels, leaf_font_size=10)
    for i, d in zip(data['icoord'], data['dcoord']):
        x = 0.5 * sum(i[1:3])
        y = d[1]
        plt.plot(x, y, 'ro')
        plt.annotate(
            '%0.6g' % y, (x, y),
            xytext=(0, -8), textcoords='offset points', va='top', ha='center')
    plt.show()


if __name__ == '__main__':
    parser = argparse.ArgumentParser(
        description='Perform hierarchical agglomerative clustering on a '
                    'CSV data set of locations, and output the steps '
                    'required to build the resulting dendrogram in CSV '
                    'format. Pass the --plot flag to see a visual.')
    subparsers = parser.add_subparsers(dest='command')

    # cluster sub-command
    p = subparsers.add_parser(
        'cluster',
        help='Perform hierarchical agglomerative clustering on a '
             'CSV data set of locations, and output the steps '
             'required to build the resulting dendrogram in CSV '
             'format. Pass the --plot flag to see a visual.')
    p.add_argument(
        'location_data', metavar='CSV_FILE',
        help='CSV with the following columns: '
             'City,Region,Country,Latitude,Longitude. '
             'Latitude and longitude should be in degrees.')
    p.add_argument(
        '--save-dist-to', metavar='FILE', type=str, default=None,
        help='Save condensed matrix to file. The format of the file is '
             'a sequence of 32-bit native floating point numbers.')
    p.add_argument(
        '--load-dist-from', metavar='FILE', type=str, default=None,
        help='Load condensed matrix from file instead of computing it.'
             'The format of the file is a sequence of 32-bit native floating '
             'point numbers.')
    p.add_argument(
        '--scipy', action='store_true',
        help='Use scipy clustering instead of fastcluster.')
    p.add_argument(
        '--method', metavar='METHOD', type=str, default='average',
        help='The clustering method to use.')
    p.add_argument(
        '--plot', action='store_true',
        help='When given, the dendrogram is plotted and show in a GUI window. '
             'Note that this can quite slow for large dendrograms.')

    # plot sub-command
    p = subparsers.add_parser(
        'plot',
        help='Plot a dendrogram given the set of steps to construct it.')
    p.add_argument(
        'location_data', metavar='CSV_FILE',
        help='CSV with the following columns: '
             'City,Region,Country,Latitude,Longitude. '
             'Latitude and longitude should be in degrees.')
    p.add_argument(
        'dendrogram_steps', metavar='CSV_FILE',
        help='The steps required to build the dendrogram in CSV format. '
             'The columns expected are cluster1,cluster2,dissimilarity,size.')

    args = parser.parse_args()

    start = time.time()
    with open(args.location_data) as rdr:
        records = parse_csv(rdr)

    if args.command == 'cluster':
        condensed = condensed_matrix_from_args(args, records)
        linkage = linkage_fast
        if args.scipy:
            linkage = linkage_sci

        start = time.time()
        steps = linkage(condensed, method=args.method)
        eprint('linkage took: %fs' % (time.time() - start))

        wtr = csv.DictWriter(
            sys.stdout,
            fieldnames=CSV_DENDROGRAM_NAMES,
            lineterminator='\n')
        wtr.writerow({name: name for name in CSV_DENDROGRAM_NAMES})
        for cluster1, cluster2, dis, size in steps:
            wtr.writerow({
                'cluster1': int(cluster1),
                'cluster2': int(cluster2),
                'dissimilarity': dis,
                'size': int(size),
            })
        if args.plot:
            plot(records, steps)
    elif args.command == 'plot':
        with open(args.dendrogram_steps) as rdr:
            steps = []
            for row in csv.DictReader(rdr):
                steps.append((
                    int(row['cluster1']),
                    int(row['cluster2']),
                    float(row['dissimilarity']),
                    int(row['size'])))
            plot(records, steps)
