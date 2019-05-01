#!/bin/env gnuplot
###############################     TERMINAL     ###############################
set terminal pngcairo dashed size 1280,720
set output './png/reset.png'

################################     TITLE     ################################
set title 'Global Reset: Increasing Number of Servers'
set title font ",22"

#################################     AXIS     #################################
#set logscale x 2
# set logscale y 2
#set yrange [0:4]
set key left top font ",20"
set ylabel 'Reset Time [s]' font ",18"
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

#################################     PLOT     #################################
plot 'test-reset_summary.csv'\
using 1:2 with linespoints lt 1 title "Reset Time",\
