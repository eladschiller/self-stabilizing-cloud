clf
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;1;2;3];
y = [;133;1221;1679];
plot(x,y, 'r-+', 'linewidth', 2);
hold on
x2 = [;1;2;3;4;5;6;7;8;9;10;11;12;13;14;15];
y2 = [;11;96;170;790;751;1465;1433;1838;2316;2214;2595;3703;4647;6128;7930];
plot(x2,y2,'b-*', 'linewidth', 2);
title({'Scalability w.r.t. number of processes.', 'The average latency per sender for a scdBroadcast, in ms.', 'Results for Local Network and PlanetLab.'})
xlabel('Number of processes')
xticks([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
ylabel('Latency for scdBroadcast in ms')
yticks([0, 500.0, 1000.0, 1500.0, 2000.0, 2500.0, 3000.0, 3500.0, 4000.0, 4500.0, 5000.0, 5500.0, 6000.0, 6500.0, 7000.0, 7500.0, 8000.0])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
saveas(gcf, ‘scd_exp1_combined_lat.pdf')
