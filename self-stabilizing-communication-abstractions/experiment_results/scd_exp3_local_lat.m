clf
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;50;100;1;20;10;20;50;10;1;100;50;1;100;10;20];
y = [;1;1;1;1;1;2;2;2;2;2;3;3;3;3;3];
z = [;592;1401;8;178;49;555;3966;1358;33;20680;12140;70;69812;700;2342];
[X,Y]=meshgrid(min(x):max(x),min(y):max(y));
Z=griddata(x,y,z,X,Y);
contour(X,Y,Z, 'linewidth', 2, 'ShowText','on');
hold on;
levels=0:1000:10000;
contour(X,Y,Z, levels, 'linewidth', 2, 'ShowText','on');
levels2=0:100:200;
contour(X,Y,Z, levels2, 'linewidth', 2, 'ShowText','on');
title({'Scalability w.r.t. bufferUnitSize.', 'The average latency per sender for a scdBroadcast, in ms.', 'Results for Local Network.'})
xlabel('BufferUnitSize')
xticks([1, 10, 20, 50, 100])
ylabel('Number of servers')
yticks([1, 2, 3])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
saveas(gcf, 'scd_exp3_local_lat_ordN.pdf')
