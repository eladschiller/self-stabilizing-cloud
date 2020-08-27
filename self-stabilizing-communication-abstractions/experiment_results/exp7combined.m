clf
set(0,'DefaultFigureVisible','off')
figure('DefaultAxesFontSize',22);
x = [;1;2;3;4;5;6;7;8;9;10];
y = [;15682;15078;16463;16794;17997;16461;18096;17976;22070;23276];
plot(x,y,'b-*' , 'linewidth', 2);
hold on
x2 = [;1;2;3];
y2 = [;975;1117;1267];
plot(x2,y2,'r-+', 'linewidth', 2);
title({'Scalability w.r.t. number of snapshotters.', 'The average latency per sender for a snapshot operation, in ms.', 'Results for PlanetLab.'})
xlabel('Number of writers')
xticks([1, 2, 3, 4, 5, 6, 7, 8, 9, 10])
ylabel('Latency for snapshot operation in ms')
set(gcf, 'PaperPosition', [0.0 0.0 15 15]);
set(gcf, 'PaperSize', [15 15]);
saveas(gcf, 'scd_exp7_combined_lat.pdf')
