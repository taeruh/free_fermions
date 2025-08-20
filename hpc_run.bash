#!/usr/bin/bash

# don't define anything before the PBS options
# don't put any directly comments behind the PBS options

#PBS -m eba
#PBS -M jannis.ruh@student.uts.edu.au
#PBS -N free_fermions

# when I do more than roughly 20 jobs, I get apparently really far down in the queue
# -> it is often faster to run less jobs but do it multiple times
#PBS -J 1-20

# 200h is the maximum, otherwise the job doesn't even get queued
#PBS -l walltime=40:00:00 
# see for max possible resource on a single node: https://hpc.research.uts.edu.au/status/
# (select=1 is probably the default (putting stuff onto one chunk(/host?)))
#PBS -l select=1:ncpus=50:mem=100GB

# this is relative to the final workdir which is ./=${PBS_O_WORKDIR}, so we don't have
# to move it from the scratch
#PBS -e ./log/
#PBS -o ./log/


bin="free_fermions"
id="${PBS_ARRAY_INDEX}"
# id="999"

cd ${PBS_O_WORKDIR}
mkdir -p log
mkdir -p output

scratch="/scratch/${USER}_${PBS_JOBID%.*}"
mkdir -p ${scratch}/output
# cp output/exact_bricks.json ${scratch}/output/
cp target/release/${bin} ${scratch}
cp arch_rust_with_sagemath.sif ${scratch}
cp -r pysrc ${scratch}

cd ${scratch}

apptainer exec arch_rust_with_sagemath.sif ./${bin} ${id}
# NOTE: `cd ${PBS_O_WORKDIR}; mv ${scratch}/output/* output` doesn't work; it's the wild
# card that makes problems in this case, but I don't know why (maybe the ${scratch} name
# is too weird)?
mv output/* ${PBS_O_WORKDIR}/output/

cd ${PBS_O_WORKDIR}
rm -rf ${scratch}
