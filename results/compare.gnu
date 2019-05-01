#!/bin/env gnuplot
###############################     TERMINAL     ###############################
set terminal pngcairo dashed size 1280,720
set output './png/comparison_with_zmq.png'

################################     TITLE     ################################
set title 'Overhead Experiment 2: Increasing Number of Servers, 1 Client'
set title font ",22"

#################################     AXIS     #################################
# set logscale x 2
# set logscale y 2
set yrange [0:1.5]
set key left top font ",20"
set ylabel 'Avg. operation latency [s]' font ",18"
set xlabel 'Number of Servers' font ",18"
set xti 5,5
set xti font ",18"
set yti font ",18"
set grid

##############################     LINE TYPE     ##############################
set linetype 1 lc rgb '#3366cc' # blue
set linetype 2 lc rgb '#dc3912' # red
set linetype 3 lc rgb '#ff9900' # yellow
set linetype 4 lc rgb '#109618' # green
set for [i=1:4] linetype i dt i ps 2.5 pt 7 lw 4

set style arrow 1 nohead filled ls 2 lc rgb 'black'

#################################     PLOT     #################################
set arrow from 17.5, graph 0 to 17.5, graph 1 as 1

plot 'test-compare_casss.csv'\
using 1:2 with linespoints lt 2 title "CASSS Read Latency",\
'' u 1:3 w linesp lt 1 title "CASSS Write Latency",\
'test-compare_cas_zmq.csv'\
using 1:2 with linespoints lt 3 title "CAS ZMQ Read Latency",\
'' u 1:3 w linesp lt 4 title "CAS ZMQ Write Latency"

#######################     NON-Self-Stabilizing CAS     #######################
set title 'Overhead Experiment 1: Increasing Number of Servers, 1 Client'
set output './png/comparison_with_cas.png'
plot 'test-compare_casss.csv'\
using 1:2 with linespoints lt 2 title "CASSS Read Latency",\
'' u 1:3 w linesp lt 1 title "CASSS Write Latency",\
'test-compare_cas.csv'\
using 1:2 with linespoints lt 3 title "CAS Read Latency",\
'' u 1:3 w linesp lt 4 title "CAS Write Latency",\
