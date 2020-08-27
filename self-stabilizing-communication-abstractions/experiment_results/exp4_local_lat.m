clf
hold on
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;0;0;0;1;2;1;1;2;3];
y = [;3;2;1;1;2;2;3;3;3];
z = [;13.366699814579906;5.386378266393534;0.8209916765371468;0.8499565774325266;5.573280317630133;5.72104792416953;13.51837625189864;13.539479379983913;13.554847622745466];
[X,Y]=meshgrid(min(x):max(x),min(y):max(y));
Z=griddata(x,y,z,X,Y);
levels=0:1:(max(z));
contour(X,Y,Z, levels, 'linewidth', 2, 'ShowText','on');
title({'Scalability w.r.t. number of corrupted processes.', 'The average latency per sender for a urbBroadcast, in ms.', 'Results for Local Network.'})
xlabel('Number of corrupted processes')
xticks([0, 1, 2, 3])
ylabel('Number of servers')
yticks([1, 2, 3])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
saveas(gcf, 'exp5_local_lat.pdf')
