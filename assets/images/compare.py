# This script compares two JSON files containing predictions from a model.
# It checks for missing file paths and mismatched keys in the predictions.
# The script takes two arguments: the test file and the reference file.
# It prints out the differences between the two files, including missing file paths
# and mismatched keys.
#
# Usage: python compare.py <test_file.json> <ref_file.json>

import json
from math import sqrt
import sys

# Thresholds for rounding the values of certain keys in the predictions
DETECTION_IGNORE = False
DETECTION_CONF_MSE_THRESHOLD = 0.1
DETECTION_BBOX_MSE_THRESHOLD = 0.1
CLASSIFICATION_IGNORE = False
CLASSIFICATION_SCORE_MSE_THRESHOLD = 0.01
PREDICTION_SCORE_MSE_THRESHOLD = 0.01
PREDICTION_SOURCE_IGNORE = False

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
    if has_detections and not DETECTION_IGNORE:
        for i, obj in enumerate(prediction['detections']):
            row['detection_%d_category' % i] = obj['category']
            row['detection_%d_conf' % i] = obj['conf']
            row['detection_%d_bbox' % i] = obj['bbox']
    has_classifications = 'classifications' in prediction
    if has_classifications and not CLASSIFICATION_IGNORE:
        for i, clss in enumerate(prediction['classifications']['classes']):
            row['classification_%d_class' % i] = clss
            row['classification_%d_score' % i] = prediction['classifications']['scores'][i]
    has_prediction = 'prediction' in prediction
    if has_prediction:
        row['prediction_class'] = prediction['prediction']
        row['prediction_score'] = prediction['prediction_score']
        if not PREDICTION_SOURCE_IGNORE:
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
detection_conf_error_sum = 0
detection_conf_error_count = 0
detection_bbox_error_sum = 0
detection_bbox_error_count = 0
classification_score_error_sum = 0
classification_score_error_count = 0
prediction_score_error_sum = 0
prediction_score_error_count = 0

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
                if key not in test:
                    #mismatched_keys.append((key, ref[key], 'missing'))
                    _ = 1
                elif key.startswith('detection_') and key.endswith('_conf'):
                    error_sum = abs(ref[key] - test[key])
                    detection_conf_error_sum += pow(error_sum, 2)
                    detection_conf_error_count += 1
                    if error_sum > DETECTION_CONF_MSE_THRESHOLD:
                        mismatched_keys.append((key, ref[key], test[key]))
                elif key.startswith('detection_') and key.endswith('_bbox'):
                    error_sum = sum(
                        abs(ref[key][i] - test[key][i]) for i in range(len(ref[key]))
                    )
                    detection_bbox_error_sum += pow(error_sum, 2)
                    detection_bbox_error_count += 1
                    if error_sum > DETECTION_BBOX_MSE_THRESHOLD:
                        mismatched_keys.append((key, ref[key], test[key]))
                elif key == 'classification_score':
                    error_sum = abs(ref[key] - test[key])
                    classification_score_error_sum += pow(error_sum, 2)
                    classification_score_error_count += 1
                    if error_sum > CLASSIFICATION_SCORE_MSE_THRESHOLD:
                        mismatched_keys.append((key, ref[key], test[key]))
                elif key == 'prediction_score':
                    error_sum = abs(ref[key] - test[key])
                    prediction_score_error_sum += pow(error_sum, 2)
                    prediction_score_error_count += 1
                    if error_sum > PREDICTION_SCORE_MSE_THRESHOLD:
                        mismatched_keys.append((key, ref[key], test[key]))
                elif ref[key] != test[key]:
                    mismatched_keys.append((key, ref[key], test[key]))

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
if detection_bbox_error_count > 0:
    print('Error (RMSE) detection bbox: %.3f' % sqrt(detection_bbox_error_sum / detection_bbox_error_count))
if detection_conf_error_count > 0:
    print('Error (RMSE) detection conf: %.3f' % sqrt(detection_conf_error_sum / detection_conf_error_count))
if classification_score_error_count > 0:
    print('Error (RMSE) classification score: %.3f' % sqrt(classification_score_error_sum / classification_score_error_count))
if prediction_score_error_count > 0:
    print('Error (RMSE) prediction score: %.3f' % sqrt(prediction_score_error_sum / prediction_score_error_count))
print()
