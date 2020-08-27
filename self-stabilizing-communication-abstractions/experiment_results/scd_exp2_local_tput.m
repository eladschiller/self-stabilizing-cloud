clf
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;1;1;2;3;1;2];
y = [;1;2;2;3;3;3];
z = [;81.6849808241065;14.405262198051805;23.953247718341185;4.0012320866426645;0.5113709921526519;2.453019357553655];
[X,Y]=meshgrid(min(x):max(x),min(y):max(y));
Z=griddata(x,y,z,X,Y);
contour(X,Y,Z, 'linewidth', 2, 'ShowText','on');
hold on
levels2=5:5:19;
contour(X,Y,Z, levels2, 'linewidth', 2, 'ShowText','on');
levels=1:2:4;
contour(X,Y,Z, levels, 'linewidth', 2, 'ShowText','on');

hold off
title({'Scalability w.r.t. number of senders.', 'The average throughput per sender,', 'in delivered scdBroadcast messages per second.', 'Results for Local Network.'})
xlabel('Number of senders')
xticks([1, 2, 3])
ylabel('Number of processes')
yticks([1, 2, 3])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
saveas(gcf, 'exp2_local_tput_final.pdf')

