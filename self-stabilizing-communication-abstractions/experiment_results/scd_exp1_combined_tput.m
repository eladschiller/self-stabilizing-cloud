clf
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;1;2;3];
y = [;49.25904913648588;9.405489478043545;3.385772254822143];
plot(x,y, 'r-+', 'linewidth', 2);
hold on
x2 = [;1;2;3;4;5;6;7;8;9;10;11;12;13;14;15];
y2 = [;315.62750543734;71.39467316881455;32.04096323408305;12.629809850882827;9.623339478331264;6.525590933813476;10.62169211110946;4.749300572505745;4.075302474242246;2.5313022542661754;1.9904961066706583;1.4513909657641983;0.7585943037471317;0.6628951953862351;0.38310261539572255];
plot(x2,y2,'b-*', 'linewidth', 2);
title({'Scalability w.r.t. number of processes.', 'The average throughput per sender, in delivered SCD messages per second.', 'Results for Local Network and PlanetLab.'})
xlabel('Number of processes')
xticks([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
ylabel('Delivered messages per second')
yticks([1.0, 10.0, 100.0, 200.0, 300.0, 350.0])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
saveas(gcf, 'scd_exp1_combined_tput.pdf')
