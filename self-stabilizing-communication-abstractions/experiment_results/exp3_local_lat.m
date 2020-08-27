clf
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;400;800;1;1000;10;100;1000;100;800;10;400;1;10;1;1000;400;100;800];
y = [;1;1;1;1;1;1;2;2;2;2;2;2;3;3;3;3;3;3];
z = [;179;486;2;716;4;37;31380;434;21226;39;6097;13;87;22;76966;16211;1308;46215];
[X,Y]=meshgrid(min(x):max(x),min(y):max(y));
Z=griddata(x,y,z,X,Y);
levels=0:3000:(max(z));
levels2=0:1000:2999;
contour(X,Y,Z,levels, 'linewidth', 2, 'ShowText','on');
hold on
contour(X,Y,Z,levels2, 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,0:200:800, 'linewidth', 2, 'ShowText','on');
title({'Scalability w.r.t. bufferUnitSize.', 'The average latency per sender for a urbBroadcast, in ms.', 'Results for Local Network.'})
xlabel('BufferUnitSize')
xticks([1, 10, 100, 400, 800, 1000])
ylabel('Number of processes')
yticks([1, 2, 3])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
hold off
saveas(gcf, 'urb_exp3_local_lat_new.pdf')
