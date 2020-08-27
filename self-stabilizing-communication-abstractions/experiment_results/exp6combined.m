clf
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;1;2;3;4;5;6;7;8;9;10];
y = [;319;429;378;616;820;1007;1636;1719;1996;2433];
plot(x,y, 'b-*', 'linewidth', 2);
hold on
x2 = [;1;2;3];
y2 = [;582;348;413];
plot(x2,y2,'r-+', 'linewidth', 2);

title({'Scalability w.r.t. number of snapshotters.', 'The average latency per sender for a write operation, in ms.', 'Results for PlanetLab.'})
xlabel('Number of snapshotters')
xticks([1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
ylabel('Latency for a write operation in ms')
yticks([100.0, 500.0, 1000.0, 1500.0, 2000.0, 2500.0])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
saveas(gcf, 'scd_exp6_combined_lat.pdf')
