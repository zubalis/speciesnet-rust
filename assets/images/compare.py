# This script compares two JSON files containing predictions from a model.
# It checks for missing file paths and mismatched keys in the predictions.
# The script takes two arguments: the test file and the reference file.
# It prints out the differences between the two files, including missing file paths
# and mismatched keys. The script also rounds the values of certain keys to a specified number of decimal places.
# The script is designed to be run from the command line and requires two JSON files as input.
# Usage: python compare.py <test_file.json> <ref_file.json>

import json
import sys

# Thresholds for rounding the values of certain keys in the predictions
DETECTION_CONF_DP = 3
DETECTION_BBOX_DP = 2
CLASSIFICATION_SCORE_DP = 3
PREDICTION_SCORE_DP = 3

# Number of errors to show in the output
MAX_ERRORS = 10

# Parse command line arguments
if len(sys.argv) != 3:
    print('Usage: python %s <test_file.json> <ref_file.json>' % sys.argv[0])
    sys.exit(1)
if not sys.argv[1].endswith('.json') or not sys.argv[2].endswith('.json'):
    print('Error: both parameters must be JSON files')
    sys.exit(1)

test_file = sys.argv[1]
ref_file = sys.argv[2]

print('Comparison:')
print('-----------')
print('Test file: %s' % test_file)
print('Ref file: %s' % ref_file)
print()

# Load the JSON files
try:
    with open(test_file, 'r') as f:
        test_data = json.load(f)
    with open(ref_file, 'r') as f:
        ref_data = json.load(f)
except FileNotFoundError as e:
    print('Error: %s' % e)
    sys.exit(1)
except json.JSONDecodeError as e:
    print('Error: %s' % e)
    sys.exit(1)

# Check if the JSON files contain the 'predictions' key
if 'predictions' not in test_data:
    print('Error: test data does not contain \'predictions\' key')
    sys.exit(1)
if 'predictions' not in ref_data:
    print('Error: ref data does not contain \'predictions\' key')
    sys.exit(1)

test_data = test_data['predictions']
ref_data = ref_data['predictions']

print('Quick look at the first prediction:')
print('-----------------------------------')
print('Test data:')
print('  %s' % test_data[0])
print('Ref data:')
print('  %s' % ref_data[0])
print()

FAILURE_MAP = {
    'DETECTOR': 'detection',
    'CLASSIFIER': 'classification',
    'PREDICTOR': 'prediction',
}

def simplify_prediction(prediction):
    row = {}
    has_detections = 'detections' in prediction
    if has_detections:
        for i, obj in enumerate(prediction['detections']):
            row['detection_%d_category' % i] = obj['category']
            row['detection_%d_conf' % i] = round(obj['conf'], DETECTION_CONF_DP)
            row['detection_%d_bbox' % i] = ",".join([str(round(x, DETECTION_BBOX_DP)) for x in obj['bbox']])
    has_classifications = 'classifications' in prediction
    if has_classifications:
        for i, clss in enumerate(prediction['classifications']['classes']):
            score = prediction['classifications']['scores'][i]
            if score < 0.0001:
                continue
            row['classification_%d_class' % i] = clss
            row['classification_%d_score' % i] = round(score, CLASSIFICATION_SCORE_DP)
    has_prediction = 'prediction' in prediction
    if has_prediction:
        row['prediction_class'] = prediction['prediction']
        row['prediction_score'] = round(prediction['prediction_score'], PREDICTION_SCORE_DP)
        row['prediction_source'] = prediction['prediction_source']
    row['detections_found'] = has_detections
    row['classifications_found'] = has_classifications
    row['predictions_found'] = has_prediction
    if 'failures' in prediction:
        for failure in prediction['failures']:
            row[FAILURE_MAP[failure] + 's_failed'] = True
        row['failures'] = "+".join(prediction['failures'])
    return row

# Compare the predictions filepath by filepath
test_data_indexed = {item['filepath']: simplify_prediction(item) for item in test_data}
ref_data_indexed = {item['filepath']: simplify_prediction(item) for item in ref_data}

missing_filepaths = []
mismatched_filepaths = []

for filepath in ref_data_indexed.keys():
    if filepath not in test_data_indexed:
        missing_filepaths.append(filepath)
        continue

    ref = ref_data_indexed[filepath]
    test = test_data_indexed[filepath]

    mismatched_keys = []

    for prefix in FAILURE_MAP.values():
        failure_key = prefix + 's_failed'
        if failure_key in ref and failure_key not in test:
            mismatched_keys.append((failure_key, "true", "missing"))
        elif failure_key not in ref and failure_key in test:
            mismatched_keys.append((failure_key, "missing", "true"))
        else:
            for key in [x for x in ref.keys() if x.startswith(prefix + '_')]:
                if key not in test or ref[key] != test[key]:
                    mismatched_keys.append((key, ref[key], test[key] if key in test else 'missing'))

    if mismatched_keys:
        mismatched_filepaths.append((filepath, mismatched_keys))


# Print the results

if len(test_data) != len(ref_data):
    print('Different number of prediction instances')
    print('  test: %d' % len(test_data))
    print('  ref: %d' % len(ref_data))
    print()

if len(missing_filepaths) > 0:
    print('Missing filepaths:')
    print('------------------')
    for filepath in missing_filepaths[:MAX_ERRORS]: # Limit the output to MAX_ERRORS
        print('Missing filepath \'%s\' in test data' % (filepath))
    if len(missing_filepaths) > MAX_ERRORS:
        print('...too many missing filepaths, only the first %d are shown' % (MAX_ERRORS))
    print()

if len(mismatched_filepaths) > 0:
    print('Mismatched filepaths:')
    print('---------------------')
    for filepath, mismatched_keys in mismatched_filepaths[:MAX_ERRORS]: # Limit the output to MAX_ERRORS
        print('Mismatched filepath \'%s\'' % (filepath))
        for key, ref_value, test_value in mismatched_keys:
            print('  %s: %s != %s' % (key, ref_value, test_value))
    if len(mismatched_filepaths) > MAX_ERRORS:
        print('...too many mismatches, only the first %d are shown' % (MAX_ERRORS))
    print()

print('Summary:')
print('--------')
print('Number of predictions in test data: %d' % (len(test_data)))
print('Number of predictions in ref data: %d' % (len(ref_data)))
print('Number of filepaths missing from test data: %d' % (len(missing_filepaths)))
print('Number of mismatched filepaths: %d out of %d' % (len(mismatched_filepaths), len(ref_data)))
