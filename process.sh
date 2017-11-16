#!/bin/bash

DIR=$1
IMG=$2

echo $1/$2

pushd $1

python3 ../demo-image-processing/image_split.py $2
 
popd