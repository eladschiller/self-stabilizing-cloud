#!/bin/env gnuplot
###############################     TERMINAL     ###############################
set terminal pngcairo dashed size 1280,720
set output './png/number_of_writers.png'

################################     TITLE     ################################
set title 'Client Scalability: Increasing Number of Writers, 10 Readers'
set title font ",22"

#################################     AXIS     #################################
set yrange [0:4]
set key left top font ",20"
set ylabel 'Avg. operation latency [s]' font ",18"
set xlabel 'Number of Writers' font ",18"
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

#################################     PLOT     #################################
plot 'test-writers_summary.csv'\
using 1:3 with linespoints lt 2 title "CASSS Read Latency",\
'' u 1:2 w linesp lt 1 tit "CASSS Write Latency",\
'test-writers_ar_summary.csv'\
using 1:3 with linespoints lt 3 title "MWMR ABD Read Latency",\
'' u 1:2 w linesp lt 4 tit "MWMR ABD Write Latency",\
