clf
hold on
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;10;50;1000;100;1;10;1;50;1000;100;100;50;10;1000;1];
y = [;1;1;1;1;1;2;2;2;2;2;3;3;3;3;3];
z = [;72;66;68;67;140;276;1347;213;132;194;510;693;1096;121;1464];
[X,Y]=meshgrid(min(x):max(x),min(y):max(y));
Z=griddata(x,y,z,X,Y);
contour(X,Y,Z, 'linewidth', 2, 'ShowText','on');
title({'Scalability w.r.t. delta.', 'The average latency per sender for a scdBroadcast, in ms.', 'Results for Local Network.'})
xlabel('Delta')
xticks([1, 10, 50, 100, 1000])
ylabel('Number of servers')
yticks([1, 2, 3])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
set(gca, 'XScale', 'log');
saveas(gcf, 'sdcd_exp4_local_lat_ordN.pdf')
