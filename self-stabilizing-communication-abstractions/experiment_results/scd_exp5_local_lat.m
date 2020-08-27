clf
hold on
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;1;2;1;1;3;2];
y = [;1;2;2;3;3;3];
z = [;128;775;690;1594;1598;1632];
[X,Y]=meshgrid(min(x):max(x),min(y):max(y));
Z=griddata(x,y,z,X,Y);
contour(X,Y,Z, 'linewidth', 2, 'ShowText','on');
title({'Scalability w.r.t. number of corrupted processes.', 'The average latency per sender for a scdBroadcast, in ms.', 'Results for Local Network.'})
xlabel('Number of corrupted processes')
xticks([1, 2, 3])
ylabel('Number of servers')
yticks([1, 2, 3])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
saveas(gcf, 'scd_exp5_local_lat_ordN.pdf')
