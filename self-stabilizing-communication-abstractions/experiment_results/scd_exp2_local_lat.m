clf
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;1;2;1;3;1;2];
y = [;1;2;2;3;3;3];
z = [;51;227;835;657;8295;552];
[X,Y]=meshgrid(min(x):max(x),min(y):max(y));
Z=griddata(x,y,z,X,Y);
levels=0:100:800;
contour(X,Y,Z, 'linewidth', 2, 'ShowText','on');
hold on
contour(X,Y,Z, levels, 'linewidth', 2, 'ShowText','on');
hold off
title({'Scalability w.r.t. number of senders.', 'The average latency per sender for a scdBroadcast, in ms.', 'Results for Local Network.'})
xlabel('Number of senders')
xticks([1, 2, 3])
ylabel('Number of processes')
yticks([1, 2, 3])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
saveas(gcf, 'exp2_local_lat_final.pdf')
