clf
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;10;50;100;1;20;100;20;10;50;1;50;10;100;1;20;1;10;50;100;20;1;20;50;100;10;100;1;10;20;50;100;50;20;1;10;1;50;10;20;100;50;100;10;1;20;20;1;50;10;100;1;10;50;100;20;20;50;100;10;1;50;20;1;100;10;20;1;100;50;10;50;1;10;100;20];
y = [;1;1;1;1;1;2;2;2;2;2;3;3;3;3;3;4;4;4;4;4;5;5;5;5;5;6;6;6;6;6;7;7;7;7;7;8;8;8;8;8;9;9;9;9;9;10;10;10;10;10;11;11;11;11;11;12;12;12;12;12;13;13;13;13;13;14;14;14;14;14;15;15;15;15;15];
z = [;10;81;299;1;23;2085;115;78;404;33;1970;169;5694;59;299;80;570;8733;13438;1573;90;1554;6145;10119;514;32101;101;890;3066;9945;62185;17338;3989;111;1200;136;22586;1860;5470;88644;35731;96273;2829;152;7472;9442;209;47677;3515;130410;215;4381;50503;155338;13721;15331;73626;187316;5531;291;91144;19636;317;191906;6817;33660;450;213174;115534;8681;139375;457;10145;218899;31085];
[X,Y]=meshgrid(min(x):max(x),min(y):max(y));
Z=griddata(x,y,z,X,Y);
contour(X,Y,Z,'linewidth', 2, 'ShowText','on');
hold on
contour(X,Y,Z,'LevelList', [0; 10; 10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [10; 30; 10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [30; 50; 10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [50;100;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [100;150;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [150;300;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [300;500;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [500;700;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [700;1000;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [1000;1500;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [1500;3000;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [2500;3000;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [5000;6000;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [6000;7000;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [7000;8000;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [8000;10000;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [10000;15000;10], 'linewidth', 2, 'ShowText','on');
title({'Scalability w.r.t. bufferUnitSize.', 'The average latency per sender for a scdBroadcast, in ms.', 'Results for PlanetLab.'})
xlabel('BufferUnitSize')
xticks([1, 10, 20, 50, 100])
ylabel('Number of processes')
yticks([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
set(gca, 'XScale', 'log')
saveas(gcf, 'scd_exp3_pl_lat.pdf')
