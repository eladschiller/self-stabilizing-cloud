clf
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;1;2;1;3;1;2;3;4;2;1;3;4;5;1;2;6;1;2;4;5;3;1;7;4;5;6;2;3;1;6;2;7;5;3;4;8;2;1;9;3;4;6;7;8;5;6;1;3;4;7;8;2;5;10;9;7;3;8;11;10;5;6;1;2;4;9;12;6;5;1;8;7;9;4;3;10;11;2;2;7;1;5;6;10;12;11;9;4;8;13;3;3;5;12;7;13;8;6;4;2;11;14;9;1;10;7;14;8;13;15;3;2;9;1;10;4;6;5;12;11];
y = [;1;2;2;3;3;3;4;4;4;4;5;5;5;5;5;6;6;6;6;6;6;7;7;7;7;7;7;7;8;8;8;8;8;8;8;8;9;9;9;9;9;9;9;9;9;10;10;10;10;10;10;10;10;10;10;11;11;11;11;11;11;11;11;11;11;11;12;12;12;12;12;12;12;12;12;12;12;12;13;13;13;13;13;13;13;13;13;13;13;13;13;14;14;14;14;14;14;14;14;14;14;14;14;14;14;15;15;15;15;15;15;15;15;15;15;15;15;15;15;15];
z = [;0;41;27;61;58;48;55;56;59;55;66;71;76;69;72;77;86;79;83;80;76;91;96;91;93;99;86;94;106;99;103;97;105;105;100;102;124;121;122;125;125;126;123;126;128;134;135;139;142;137;133;139;137;141;135;157;158;152;154;156;158;148;153;155;148;156;162;165;163;161;159;161;163;162;155;161;165;160;184;183;181;187;180;183;182;183;183;196;190;185;186;207;198;223;205;221;204;223;197;219;204;202;208;232;201;265;229;260;234;246;232;222;236;225;236;250;232;321;257;225];
[X,Y]=meshgrid(min(x):max(x),min(y):max(y));
Z=griddata(x,y,z,X,Y);
contour(X,Y,Z, 'LevelList', [0;10;10], 'linewidth', 2, 'ShowText','on');
hold on
contour(X,Y,Z, 'LevelList', [10;20;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [20;30;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [30;40;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [40;50;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [50;60;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [60;70;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [70;80;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [80;90;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [90;100;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [100;110;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [110;120;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [120;130;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [130;140;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [140;150;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [150;160;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [160;170;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [170;180;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [180;190;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [190;200;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [200;220;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [220;230;10], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z, 'LevelList', [230;250;10], 'linewidth', 2, 'ShowText','on');
title({'Scalability w.r.t. number of corrupted processes.', 'The average latency per sender for a urbBroadcast, in ms.', 'Results for PlanetLab.'})
xlabel('Number of corrupted processes')
xticks([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
ylabel('Number of processes')
yticks([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
saveas(gcf, ‘exp5_pl_lat_new.pdf')
