#!/system/bin/sh
HERE="$(cd "$(dirname "$0")" && pwd)"

export CLASSPATH="$HERE/main.jar"
export ANDROID_DATA="$HERE"
export LD_LIBRARY_PATH="$HERE"

cmd="app_process $HERE com.jhc.Main $@" # app_process32 ?
echo "run: $cmd"
exec $cmd
