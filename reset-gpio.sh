#!/bin/bash
for d in `ls -d /sys/class/gpio/gpio*`; do
  pin=`basename $d`
  echo ${pin#gpio} >/sys/class/gpio/unexport 2>/dev/null
done
