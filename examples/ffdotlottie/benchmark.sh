#!/bin/bash

cpu_cores=$(sysctl -n hw.ncpu)

start_time=$(date +%s)

./.bin/macos-arm64/ffdotlottie --input ./assets/cartoon.json --output ./output.mp4 --width 512 --height 512 --background-color "#00FFFFFF" --fps 30 &
pid=$!

peak_cpu=0
peak_mem=0

while ps -p $pid > /dev/null; do
  timestamp=$(date +"%Y-%m-%d %H:%M:%S")

  current_usage=$(ps -o %cpu=,%mem=,rss= -p $pid)
  
  current_cpu=$(echo $current_usage | awk '{print $1}')
  current_mem_kb=$(echo $current_usage | awk '{print $3}')

  current_mem_mb=$(echo "scale=2; $current_mem_kb/1024" | bc)

  total_cpu_usage=$(echo "scale=2; $current_cpu / $cpu_cores" | bc)

  printf "[$timestamp] Total CPU usage: %.2f%%, Memory (RSS): %.2f MB\n" "$total_cpu_usage" "$current_mem_mb"

  peak_cpu=$(echo "$total_cpu_usage $peak_cpu" | awk '{if ($1 > $2) print $1; else print $2}')
  peak_mem=$(echo "$current_mem_mb $peak_mem" | awk '{if ($1 > $2) print $1; else print $2}')

  sleep 0.1
done

end_time=$(date +%s)

total_runtime=$((end_time - start_time))

runtime_hms=$(printf '%02d:%02d:%02d\n' $((total_runtime/3600)) $((total_runtime%3600/60)) $((total_runtime%60)))

printf "Peak total CPU usage: %.2f%%\n" "$peak_cpu"
printf "Peak Memory usage: %.2f MB\n" "$peak_mem"
printf "Total runtime: %s\n" "$runtime_hms"
