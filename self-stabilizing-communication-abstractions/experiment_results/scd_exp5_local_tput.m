clf
hold on
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;1;1;2;1;2;3];
y = [;1;2;2;3;3;3];
z = [;50.880463700378506;13.205807505528071;8.398660650273182;2.7064905958353527;3.8703709312302785;2.842727787115748];
[X,Y]=meshgrid(min(x):max(x),min(y):max(y));
Z=griddata(x,y,z,X,Y);
contour(X,Y,Z, 'linewidth', 2, 'ShowText','on');
title({'Scalability w.r.t. number of corrupted processes.', 'The average throughput per sender, in delivered SCD messages per second.', 'Results for Local Network.'})
xlabel('Number of corrupted processes')
xticks([1, 2, 3])
ylabel('Number of servers')
yticks([1, 2, 3])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
saveas(gcf, 'scd_exp5_local_tput_ordN.pdf')
