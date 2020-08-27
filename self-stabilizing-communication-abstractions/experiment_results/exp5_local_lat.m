clf
hold on
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;1;1;1;10000;10;1000;50;100;100;10;10000;50;1000;50;1000;10000;10;100];
y = [;1;2;3;1;1;1;1;1;2;2;2;2;2;3;3;3;3;3];
z = [;5;40;90;2;3;2;2;2;31;33;31;32;31;68;68;67;71;67];
[X,Y]=meshgrid(min(x):max(x),min(y):max(y));
Z=griddata(x,y,z,X,Y);
levels=(min(z)):0.0008:(max(z));
contour(X,Y,Z, 'linewidth', 2, 'ShowText','on');
title({'Scalability w.r.t. delta.', 'The average latency per sender for a urbBroadcast, in ms.', 'Results for Local Network.'})
xlabel('Delta')
xticks([1, 10, 100, 1000, 10000])
ylabel('Number of servers')
yticks([1, 2, 3])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
set(gca, 'XScale', 'log');
saveas(gcf, 'exp4_local_lat.pdf')
