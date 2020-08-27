clf
hold on
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;1;2;1;3;1;2];
y = [;1;2;2;3;3;3];
z = [;0.8209916765371468;5.386378266393534;3.5036715519046537;13.366699814579906;6.37318028671573;9.596460159833088];
[X,Y]=meshgrid(min(x):max(x),min(y):max(y));
Z=griddata(x,y,z,X,Y);
levels=0:1:(max(z));
contour(X,Y,Z,levels, 'linewidth', 2, 'ShowText','on');
title({'Scalability w.r.t. number of senders.', 'The average latency per sender for a urbBroadcast, in ms.', 'Results for Local Network.'})
xlabel('Number of senders')
xticks([1, 2, 3])
ylabel('Number of processes')
yticks([1, 2, 3])
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
saveas(gcf, 'urb_exp2_local_lat.pdf')


