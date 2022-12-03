clear all 
clc 
close all 
%DATOS INICIALES
AB=0.5; BC=1.0; phi=pi/4;
%CÁLCULO DE LAS POSICIONES
xA=0; 
yA=0;
yC=0;
%Puntos de referencia iniciales
xCref=1.29;
%Se define el paso de la simulación
Paso=pi/4;
for I=phi:Paso:phi+2*pi
    xB=AB*cos(I);
    yB=AB*sin(I);
    %Para el cálculo de la posición faltante
    [ xC1,yC1,xC2,yC2 ] = lincir( xA,yA,AB+BC,yA,xB,yB,BC);
    % Se escoge una de las dos soluciones
    [ xC,yC ] = distMinima( xCref,yC,xC1,yC1,xC2,yC2); 
    figure(1);
    hold on
    plot([xA,xB,xC],[yA,yB,yC],'b-o');
    title('Simulación de las posiciones')
    xlabel('x (m)')
    ylabel('y (m)')
    axis([-0.6 1.6 -0.6 0.8]);
    grid
    %Se actualiza la refrencia a la última coordenada calculada
    xCref=xC;
    pause(0.01)    
    hold off
end

