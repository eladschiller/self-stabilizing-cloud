clf
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;100;500;1;500;100;1;1;100;500;1;100;500;100;1;500;100;500;1;1;100;500;100;1;500;1;100;500;100;1;500;1;100;500;500;1;100;1;500;100;100;1;500;100;500;1;1000;1000;1000;1000;1000;1000;1000;1000;1000;1000;1000;1000;1000;1000;1000;10;10;10;10;10;10;10;10;10;10;10;10;10;10;10;50;5;5;50;50;5;50;5;50;5;5;50;5;50;5;50;5;50;50;5;50;5;50;5;5;50;5;50;50;5];
y = [;1;1;1;2;2;2;3;3;3;4;4;4;5;5;5;6;6;6;7;7;7;8;8;8;9;9;9;10;10;10;11;11;11;12;12;12;13;13;13;14;14;14;15;15;15;1;2;3;4;5;6;7;8;9;10;11;12;13;14;15;1;2;3;4;5;6;7;8;9;10;11;12;13;14;15;1;1;2;2;3;3;4;4;5;5;6;6;7;7;8;8;9;9;10;10;11;11;12;12;13;13;14;14;15;15];
z = [;6;48;0;697;75;59;40;141;1228;83;263;3553;197;44;1766;257;2128;74;39;297;5481;1443;58;51472;36;232;3461;280;33;2229;50;292;3002;2752;45;302;35;3174;380;470;36;3718;426;4135;46;126;2568;8309;17193;31040;38608;54346;193199;76464;88900;86797;336092;107156;94247;337263;0;27;56;59;69;78;96;101;129;135;156;166;209;205;246;3;0;30;54;105;58;138;40;161;71;56;174;56;256;72;368;71;299;280;74;328;67;1551;108;79;359;79;420;3911;150];
[X,Y]=meshgrid(min(x):max(x),min(y):max(y));
Z=griddata(x,y,z,X,Y);
contour(X,Y,Z, 'linewidth', 2, 'ShowText','on');
hold on
contour(X,Y,Z, 'LevelList', [0;20;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [20;40;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [40;50;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [50;70;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [100;200;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [200;400;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [400;700;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [700;1000;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [1000;2000;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [2000;2000;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [5000;20000;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [20000;30000;10], 'linewidth', 2, 'ShowText','on');
title({'Scalability w.r.t. bufferUnitSize.', 'The average latency per sender for a urbBroadcast, in ms.', 'Results for PlanetLab.'})
xlabel('BufferUnitSize')
xticks([1, 5, 50, 100, 500, 1000, 10000])
ylabel('Number of processes')
yticks([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
set(gca, 'XScale', 'log')
saveas(gcf, 'exp3_pl_lat_new.pdf')
