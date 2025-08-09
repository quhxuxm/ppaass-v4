#!/bin/sh
a=0
while [ $a -lt 5 ];
do
    process_id=$(ps -ef | grep "ppaass-agent" | grep -v grep | awk '{print $2}')
    if [ -z $process_id ]; then
        echo "No ppaass-agent process"
    else
        echo "Found ppaass-agent process: $process_id"
        kill -9 $process_id
        echo "Kill ppaass-agent process: $process_id"
        break
    fi
    a=`expr $a + 1`
    sleep 2
done
ulimit -n 65536
sudo nohup ./concrete-start-agent.sh >run.log 2>&1 &