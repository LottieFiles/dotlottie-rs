#!/bin/bash

./.bin/macos-arm64/ffdotlottie --input ./assets/cartoon.json --output ./output.mp4 --width 512 --height 512 &
pid=$!

peak_cpu=0
peak_mem=0

log_file="./data.log"
> $log_file

while ps -p $pid > /dev/null; do
  current_usage=$(ps -o %cpu=,%mem=,rss= -p $pid)
  
  current_cpu=$(echo $current_usage | awk '{print $1}')
  current_mem_kb=$(echo $current_usage | awk '{print $3}')

  current_mem_mb=$(echo "scale=2; $current_mem_kb/1024" | bc)

  echo "CPU: $current_cpu%, Memory (RSS): $current_mem_mb MB" >> $log_file

  peak_cpu=$(echo "$current_cpu $peak_cpu" | awk '{if ($1 > $2) print $1; else print $2}')

  peak_mem=$(echo "$current_mem_mb $peak_mem" | awk '{if ($1 > $2) print $1; else print $2}')

  sleep 0.1
done

echo "Peak CPU usage: $peak_cpu%" | tee -a $log_file
echo "Peak Memory usage: $peak_mem MB" | tee -a $log_file
