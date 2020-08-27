clf
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;1;1;2;1;2;3;4;2;1;3;3;1;2;5;4;3;2;5;1;6;4;7;3;1;5;6;2;4;1;4;5;2;3;8;6;7;3;7;6;5;8;4;1;2;9;2;5;6;9;10;4;8;3;7;1;11;7;1;2;10;8;5;6;3;4;9;11;4;6;9;12;8;2;5;7;10;1;3;5;2;6;13;7;11;8;1;10;3;12;4;9;9;5;2;3;1;11;13;7;6;12;10;8;4;14;3;12;8;6;9;1;7;14;13;15;5;10;11;2;4];
y = [;1;2;2;3;3;3;4;4;4;4;5;5;5;5;5;6;6;6;6;6;6;7;7;7;7;7;7;7;8;8;8;8;8;8;8;8;9;9;9;9;9;9;9;9;9;10;10;10;10;10;10;10;10;10;10;11;11;11;11;11;11;11;11;11;11;11;12;12;12;12;12;12;12;12;12;12;12;12;13;13;13;13;13;13;13;13;13;13;13;13;13;14;14;14;14;14;14;14;14;14;14;14;14;14;14;15;15;15;15;15;15;15;15;15;15;15;15;15;15;15];
z = [;0;45;29;46;51;51;55;45;34;48;51;53;57;73;62;53;51;71;56;79;64;97;67;67;86;95;55;80;54;81;89;70;81;108;86;89;71;113;100;95;120;91;64;81;131;71;101;105;132;136;87;124;75;112;48;154;120;94;65;152;134;103;105;86;94;141;168;93;116;142;165;137;77;102;124;144;131;87;110;78;127;183;133;164;146;55;162;90;174;103;152;165;126;97;122;100;179;210;149;145;186;183;149;113;213;101;217;159;146;185;119;158;263;213;278;151;187;184;92;132];
[X,Y]=meshgrid(min(x):max(x),min(y):max(y));
Z=griddata(x,y,z,X,Y);
contour(X,Y,Z,'LevelList', [10;20;10], 'linewidth', 2, 'ShowText','on');
hold on
contour(X,Y,Z,'LevelList', [10;20;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [20;30;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [30;40;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [50;60;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [60;70;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [70;80;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [90;100;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [100;110;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [110;120;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [120;130;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [130;140;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [150;170;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [170;180;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [180;200;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [200;220;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [220;240;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [240;250;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [250;260;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [260;280;10], 'linewidth', 2, 'ShowText','on');
title({'Scalability w.r.t. number of senders.', 'The average latency per sender for a urbBroadcast, in ms.', 'Results for PlanetLab.'})
xlabel('Number of senders')
xticks([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
ylabel('Number of processes')
yticks([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
saveas(gcf, 'exp2_pl_lat_new.pdf')
