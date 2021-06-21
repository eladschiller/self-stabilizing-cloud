clf
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;1;2;1;1;2;3;3;4;2;1;5;2;3;4;1;4;2;1;3;6;5;4;5;2;6;7;1;3;8;4;5;6;7;3;1;2;5;8;2;1;3;6;9;7;4;5;3;8;6;1;10;4;7;2;9;9;11;6;8;2;3;4;5;1;10;7;12;1;9;3;2;8;5;4;7;10;11;6;10;13;12;4;2;6;11;7;3;5;1;8;9;4;9;1;2;10;12;11;3;8;14;6;5;13;7;3;14;9;15;8;10;1;5;7;11;2;4;6;12;13];
y = [;1;2;2;3;3;3;4;4;4;4;5;5;5;5;5;6;6;6;6;6;6;7;7;7;7;7;7;7;8;8;8;8;8;8;8;8;9;9;9;9;9;9;9;9;9;10;10;10;10;10;10;10;10;10;10;11;11;11;11;11;11;11;11;11;11;11;12;12;12;12;12;12;12;12;12;12;12;12;13;13;13;13;13;13;13;13;13;13;13;13;13;14;14;14;14;14;14;14;14;14;14;14;14;14;14;15;15;15;15;15;15;15;15;15;15;15;15;15;15;15];
z = [;9;78;75;108;150;155;381;548;240;111;487;198;287;383;144;536;243;168;366;937;752;509;760;333;899;1209;211;421;2077;860;1118;1243;1607;580;388;445;1006;2161;463;471;640;1348;2538;1672;805;1416;740;2584;1642;509;3261;943;2041;485;3248;3220;4486;1959;2728;512;820;1073;1472;740;3796;2151;5208;629;3528;925;640;3226;1474;1267;2692;3903;4599;2068;4414;6526;5708;1262;686;1980;4613;2846;851;1808;865;3416;3740;1434;3975;849;757;4491;6138;5342;1108;4424;8631;2174;2001;7258;2921;974;8883;4775;10314;3694;5389;1303;2205;2896;6396;973;1539;2525;6492;7730];
[X,Y]=meshgrid(min(x):max(x),min(y):max(y));
Z=griddata(x,y,z,X,Y);
contour(X,Y,Z, 'linewidth', 2, 'ShowText','on');
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
title({'Scalability w.r.t. number of senders.', 'The average latency per sender for a scdBroadcast, in ms.', 'Results for PlanetLab.'})
xlabel('Number of senders')
xticks([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
ylabel('Number of processes')
yticks([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
saveas(gcf, 'scd_exp2_pl_lat.pdf')