clf
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;10;50;100;1000;1;1;1000;50;10;100;100;10;1000;50;1;50;10;1;1000;100;100;50;1000;10;1;50;1000;100;10;1;100;10;1;1000;50;10;1000;100;50;1;1;1000;50;100;10;1000;10;1;50;100;1;10;1000;100;50;50;1000;10;1;100;10;1;1000;100;50;50;1;100;1000;10;50;100;1;1000;10];
y = [;1;1;1;1;1;2;2;2;2;2;3;3;3;3;3;4;4;4;4;4;5;5;5;5;5;6;6;6;6;6;7;7;7;7;7;8;8;8;8;8;9;9;9;9;9;10;10;10;10;10;11;11;11;11;11;12;12;12;12;12;13;13;13;13;13;14;14;14;14;14;15;15;15;15;15];
z = [;12;10;9;9;35;109;64;76;125;62;150;209;158;168;275;609;472;813;3142;842;515;486;489;675;1119;988;1010;956;1741;2655;1114;2069;3181;1224;1293;3490;1654;1826;1964;4422;4183;2109;2861;2195;3387;3123;4214;5727;3677;3494;5292;4377;4343;3978;4394;5828;5135;5843;6988;5360;6504;8847;6508;5967;7352;7497;9146;8017;7541;8457;9205;9331;10141;9244;8801];
[X,Y]=meshgrid(min(x):max(x),min(y):max(y));
Z=griddata(x,y,z,X,Y);
contour(X,Y,Z, 'linewidth', 2, 'ShowText','on');
hold on
contour(X,Y,Z,'LevelList', [0; 10; 10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [10; 50; 10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [50;100;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [100;300;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [300;500;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [500;700;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [700;1000;10], 'linewidth', 2, 'ShowText','on');
title({'Scalability w.r.t. delta.', 'The average latency per sender for a scdBroadcast, in ms.', 'Results for PlanetLab.'})
xlabel('Delta')
xticks([1, 10, 50, 100, 1000])
ylabel('Number of processes')
yticks([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
set(gca, 'XScale', 'log')
saveas(gcf, ‘scd_exp4_pl_lat.pdf')
