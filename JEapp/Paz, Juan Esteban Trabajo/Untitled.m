figure();
hold off
title('Gráfica de posiciones')
xlabel('x (m)')
ylabel('y (m)')
grid

forcef=0.1;
for J=1:size(xAv)
    plot([xAv(J),xBv(J),xCv(J)],[yAv(J),yBv(J),yCv(J)],'r-o');
    hold on
    plot([xFv(J),xBv(J),xEv(J)],[yFv(J),yBv(J),yEv(J)],'g-o');
    plot([xDv(J),xEv(J)],[yDv(J),yEv(J)],'b-o');
    plot([xCv(J),xGv(J)],[yCv(J),yGv(J)],'c-o');
    text(xAv(J),yAv(J),' A'); 
    text(xBv(J),yBv(J),' B');
    text(xCv(J),yCv(J),' C');
    text(xDv(J),yDv(J),' D'); 
    text(xEv(J),yEv(J),' E'); 
    text(xFv(J),yFv(J),' F');
    text(xGv(J),yGv(J),' G');
    quiver3(xBv(J),yBv(J),0,F32s(J,1)*forcef,F32s(J,2)*forcef,0);%F32
    text(xBv(J)+F32s(J,1)*forcef,yBv(J)+F32s(J,2)*forcef,'F32'); 
    quiver3(xCv(J),yCv(J),0,F52s(J,1)*forcef,F52s(J,2)*forcef,0);%F52
    text(xCv(J)+F52s(J,1)*forcef,yCv(J)+F52s(J,2)*forcef,'F52');
    quiver3(xEv(J),yEv(J),0,F43s(J,1)*forcef,F43s(J,2)*forcef,0);%F43
    text(xEv(J)+F43s(J,1)*forcef,yEv(J)+F43s(J,2)*0.01,'F43');
    quiver3(xFv(J),yFv(J),0,F63s(J,1)*forcef,F63s(J,2)*forcef,0);%F63
    text(xFv(J)+F63s(J,1)*forcef,yFv(J)+F63s(J,2)*forcef,'F63');
    quiver3(xCv(J),yCv(J),0,F52s(J,1)*0.01,F52s(J,2)*0.01,0);%F52
    text(xCv(J)+F52s(J,1)*forcef,yCv(J)+F52s(J,2)*forcef,'F52');
    quiver3(xGv(J),yGv(J),0,F65s(J,1)*0.01,F65s(J,2)*0.01,0);%F65
    text(xGv(J)+F65s(J,1)*forcef,yGv(J)+F65s(J,2)*forcef,'F65');
    axis([-1 4 -1 4]);
    axis manual;
    hold off
    pause(0.05)
    J=J+1;
end
