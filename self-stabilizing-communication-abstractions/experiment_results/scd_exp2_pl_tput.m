clf
hold on
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;1;1;2;3;1;2;2;1;4;3;5;2;1;3;4;1;5;3;6;2;4;3;4;5;6;7;2;1;2;8;4;1;7;6;5;3;6;3;2;1;4;5;9;7;8;10;9;1;4;2;6;7;8;3;5;9;10;5;8;7;3;2;4;11;1;6;11;10;8;5;12;2;1;6;7;4;9;3;7;13;8;5;4;9;12;2;6;11;3;10;1;2;11;14;9;1;7;5;4;12;8;3;13;6;10;8;12;6;11;14;15;5;4;13;3;7;1;10;2;9];
y = [;1;2;2;3;3;3;4;4;4;4;5;5;5;5;5;6;6;6;6;6;6;7;7;7;7;7;7;7;8;8;8;8;8;8;8;8;9;9;9;9;9;9;9;9;9;10;10;10;10;10;10;10;10;10;10;11;11;11;11;11;11;11;11;11;11;11;12;12;12;12;12;12;12;12;12;12;12;12;13;13;13;13;13;13;13;13;13;13;13;13;13;14;14;14;14;14;14;14;14;14;14;14;14;14;14;15;15;15;15;15;15;15;15;15;15;15;15;15;15;15];
z = [;360.19113853227367;67.9609673389438;90.29132324719579;21.916441544121252;13.411277147395747;14.230226995568849;8.451050921076751;9.121154935180492;10.702907681013984;9.459435941562965;9.776864354794753;7.736898568691988;7.580361944136714;8.819678368653788;7.638211056661425;6.3310255878422526;4.130291123520886;6.502630492719214;5.509155704371996;7.132977253950082;6.026943849511891;4.8777265800140714;5.494227541856444;3.9761694154474143;5.104078371290394;3.9410582465314032;5.216616457266558;5.241362840931111;3.742320929043972;3.175167018716958;2.5894706709571;3.79862848911657;2.4729839952439314;2.642691095892629;2.1266209799249287;4.457162270207086;2.7376206169973485;2.907099141699299;3.3803383401638722;2.615853654524135;3.456628435206284;3.4296630851990044;1.853571317408911;3.130426526586403;1.9658616278161318;1.7586506727157531;1.3329867182072865;2.8109141874381574;3.480865640466314;3.1495214616587655;2.679555541740357;1.3132772684666831;1.3409032920373265;3.032439645649719;2.6040294888644326;0.6836517173365755;0.37726154295853154;1.565506499545286;1.0459439763689615;1.615690020010394;2.8276265825131555;2.9625547716353937;1.6792204504556951;0.9723373317660068;2.1817134795628177;0.4902359148050027;0.48702869532836923;0.4271972552513187;0.8767572856068764;1.1334652744656444;0.6178039697483582;2.767006561644562;2.3730175057333702;1.4853953817655827;0.5688329103303862;1.546404245057517;0.7444020811927923;2.511990809710637;0.35100229271404915;0.06839531468464857;0.2830596465916632;0.4596644779687776;2.334554649022153;0.4903367086774736;0.26542711717894096;2.696116142068599;1.2080526529983706;0.4657728188496095;2.134935911384319;0.31495101768306416;1.916611632704916;1.777091988471542;0.3119157464929935;0.13402851056240295;0.25861171037133324;1.5614047918264873;0.3690309868593631;0.3608513332062807;1.8475992878406635;0.21714785836477085;0.5803151750175728;1.861924001159664;0.20738867133403815;0.8769161889254282;0.16992244953677077;0.11064070331506767;0.19024339597239176;0.505739859145328;0.23372319530331126;0.040018685012833094;0.17401842041648632;0.4999717913000229;0.25186554174854064;0.09238802392403017;1.7987305891796552;0.05081151933809759;1.1698413887845063;0.2966288642042059;1.7760247890205603;0.2457412450702721];
[X,Y]=meshgrid(min(x):max(x),min(y):max(y));
Z=griddata(x,y,z,X,Y);
contour(X,Y,Z, 'linewidth', 2, 'ShowText','on');
hold on
contour(X,Y,Z,'LevelList', [0;0.03;0.01], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [0.03;0.05;0.01], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [0.05;0.1;0.01], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [0.1;0.3;0.1], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [0.3;0.5;0.1], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [0.5;1;0.1], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [1;1.5;1], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [1.5;2;1], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [2;3;1], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [3;4;1], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [4;6;1], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [6;8;1], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [8;10;1], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [10;20;1], 'linewidth', 2, 'ShowText','on');
contour(X,Y,Z,'LevelList', [40;50;1], 'linewidth', 2, 'ShowText','on');
title({'Scalability w.r.t. bufferUnitSize.', 'The average throughput per sender,’ ,  ‘in delivered scdBroadcast messages per second.', 'Results for PlanetLab.'})
xlabel('Number of senders')
xticks([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
ylabel('Number of processes')
yticks([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
saveas(gcf, 'scd_exp2_pl_tput.pdf')