clf
hold on
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;100;10;50;20;1;50;1;100;20;10;20;50;100;1;10];
y = [;1;1;1;1;1;2;2;2;2;2;3;3;3;3;3];
z = [;33.42788926384419;83.61731590684776;37.21744607717396;47.76345921342621;48.40526009271341;9.004648458198103;14.472181443188935;4.611006164271243;19.59830220957127;19.917524759098473;2.9616099556867326;0.04797714283224758;0.035602424462181044;3.150560051192354;3.7573007888593337];
[X,Y]=meshgrid(min(x):max(x),min(y):max(y));
Z=griddata(x,y,z,X,Y);
contour(X,Y,Z, 'linewidth', 2, 'ShowText','on');
title({'Scalability w.r.t. bufferUnitSize.', 'The average throughput per sender, in delivered SCD messages per second.', 'Results for Local Network.'})
xlabel('BufferUnitSize')
xticks([1, 10, 20, 50, 100])
ylabel('Number of servers')
yticks([1, 2, 3])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
set(gca, 'XScale','log');
saveas(gcf, 'scd_exp3_local_tput_ordN.pdf')
