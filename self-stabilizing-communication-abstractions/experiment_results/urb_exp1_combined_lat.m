clf
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;1;2;3];
y = [;0.8554573236594786;5.862378758518694;13.516713898399098];
plot(x,y, 'r-+', 'linewidth', 2);
hold on
x2 = [;1;2;3;4;5;6;7;8;9;10;11;12;13;14;15];
y2 = [;0;27;56;59;69;78;96;101;129;135;156;166;209;205;246];
plot(x2,y2,'b-*', 'linewidth', 2);
title({'Scalability w.r.t. number of processes.', 'The average latency per sender for a urbBroadcast, in ms.', 'Results for Local Network and PlanetLab.'})
xlabel('Number of processes')
xticks([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
ylabel('Latency for urbBroadcast in ms')
yticks([1.0, 10.0, 100.0,200.0, 250.0, 1000.0])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
saveas(gcf, 'exp1_pl_lat_new.pdf')
