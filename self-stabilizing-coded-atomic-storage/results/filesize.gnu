#!/bin/env gnuplot
###############################     TERMINAL     ###############################
set terminal pngcairo dashed size 1280,720
set output './png/filesize.png'

################################     TITLE     ################################
set title 'Data Object Scalability: Increasing Size, 1 Writer'
set title font ",22"

#################################     AXIS     #################################
set logscale x 2
# set logscale y 2
set yrange [0:]
set key left top font ",20"
set ylabel 'Avg. operation latency [s]' font ",18"
set xlabel 'Data Object Size [KiB]' font ",18"
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
set multiplot
set object 1 rect from 3,6 to 256,20 lw 5
set object 1 rect fc rgb 'white' fillstyle solid 0.0 noborder

plot 'test-filesize_summary.csv'\
using 1:2 with linespoints lt 2 title "CASSS Read Latency",\
'' u 1:3 w linesp lt 1 title "CASSS Write Latency",\
'test-filesize_ar_summary.csv'\
using 1:2 with linespoints lt 3 title "MWMR ABD Read Latency",\
'' u 1:3 w linesp lt 4 title "MWMR ABD Write Latency",\

###############################     ZOOM BOX     ###############################
set origin 0.1,0.2
set size 0.6,0.45
set xrange [1:4096]
set yrange [0:3.5]
unset key
unset xlabel
unset ylabel
unset title

plot 'test-filesize_summary.csv'\
using 1:2 with linespoints lt 2,\
'' u 1:3 w linesp lt 1,\
'test-filesize_ar_summary.csv'\
using 1:2 with linespoints lt 3,\
'' u 1:3 w linesp lt 4
