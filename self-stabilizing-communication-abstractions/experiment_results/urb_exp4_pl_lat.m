clf
hold on
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;100;10;50;1000;50;10;100;1000;100;50;10;1000;10;100;1000;50;100;1000;50;10;10;1000;100;50;10;1000;100;50;10;100;1000;50;100;50;10;1000;10;100;50;1000;1000;100;50;10;10;50;100;1000;1000;10;100;50;1000;50;100;10;1000;50;10;100;10000;10000;10000;10000;10000;10000;10000;10000;10000;10000;10000;10000;10000;10000;10000;500;1;1;500;1;500;500;1;500;1;1;500;1;500;1;500;500;1;500;1;1;500;500;1;1;500;1;500;500;1];
y = [;1;1;1;1;2;2;2;2;3;3;3;3;4;4;4;4;5;5;5;5;6;6;6;6;7;7;7;7;8;8;8;8;9;9;9;9;10;10;10;10;11;11;11;11;12;12;12;12;13;13;13;13;14;14;14;14;15;15;15;15;1;2;3;4;5;6;7;8;9;10;11;12;13;14;15;1;1;2;2;3;3;4;4;5;5;6;6;7;7;8;8;9;9;10;10;11;11;12;12;13;13;14;14;15;15];
z = [;0;0;0;0;27;30;19;14;33;49;45;36;51;50;52;54;60;67;66;44;60;66;61;53;49;69;71;60;100;94;97;93;61;54;59;49;67;74;62;80;85;86;70;57;85;80;87;88;104;80;85;72;103;80;94;76;109;84;69;108;0;31;54;54;71;73;75;97;114;116;127;149;143;167;245;0;0;57;46;74;57;55;52;54;62;74;71;70;77;104;100;107;119;111;120;130;116;145;165;152;133;155;142;222;253];
[X,Y]=meshgrid(min(x):max(x),min(y):max(y));
Z=griddata(x,y,z,X,Y);
contour(X,Y,Z,'LevelList', [10;20;10], 'linewidth', 2, 'ShowText','on');
hold on
contour(X,Y,Z,'LevelList', [10;20;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [20;30;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [30;40;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [50;55;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [55;60;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [60;70;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [70;80;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [90;100;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [100;110;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [110;120;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [120;130;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [130;150;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [150;180;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [180;200;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [200;220;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [220;240;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [240;250;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [250;260;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [260;280;10], 'linewidth', 2, 'ShowText','on');
title({'Scalability w.r.t. delta.', 'The average latency per sender for a urbBroadcast, in ms.', 'Results for PlanetLab.'})
xlabel('Delta')
xticks([1, 10, 50, 100, 500,1000, 10000])
ylabel('Number of processes')
yticks([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
set(gca, 'XScale', 'log')
saveas(gcf, 'exp4_pl_lat_new.pdf')
