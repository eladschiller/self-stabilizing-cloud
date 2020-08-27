clf
hold on
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;50;1000;10;100;1;1000;1;50;10;100;100;10;1000;50;1];
y = [;1;1;1;1;1;2;2;2;2;2;3;3;3;3;3];
z = [;66.91755047609506;62.27529010089339;59.99325442306848;65.97754716802476;45.53860780877146;12.012161845792443;7.951288978958116;21.955334336354255;24.748761950790225;18.953635324929145;1.8066728166919568;5.47630826236352;0.06156543383151867;3.6543278228170144;3.1301561440676364];
[X,Y]=meshgrid(min(x):max(x),min(y):max(y));
Z=griddata(x,y,z,X,Y);
contour(X,Y,Z, 'linewidth', 2, 'ShowText','on');
title({'Scalability w.r.t. delta.', 'The average throughput per sender, in delivered SCD messages per second.', 'Results for Local Network.'})
xlabel('Delta')
xticks([1, 10, 50, 100, 1000])
ylabel('Number of servers')
yticks([1, 2, 3])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
set(gca, 'XScale','log');
saveas(gcf, 'scd_exp4_local_tput_ordN.pdf')
