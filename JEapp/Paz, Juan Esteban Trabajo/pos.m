clear all 
clc 
close all 
%DATOS INICIALES
AB=0.5; BC=0.6; %%barra2
DE=0.6;%barra4
yAD=-0.5;xAD=0.2;%1
BE=0.5;EF=1.2; %barra3
FG=0.6;GC=0.5;%barra5
phi=pi/4;
%CÁLCULO DE LAS POSICIONES
xA=0; 
yA=0;
y=0;
xD=xA+xAD; 
yD=yA+yAD;
%Puntos de referencia iniciales
xEref=DE;
yFref=BE;
xFref=xAD;
xGref=AB;
%Calculo Posiciones con datos conocidos
xB=AB*cos(phi);
yB=AB*sin(phi);
xC=(AB+BC)*cos(phi);
yC=(AB+BC)*sin(phi);
%Calculo de posiciones desconocidas

    %punto E circulo rDE,c rBE
    %Para el cálculo de la posición faltante
    [ xE1,yE1,xE2,yE2 ] = circir( xB,yB,BE,xD,yD,BE);
    % Se escoge una de las dos soluciones
    [ xE,yE ] = distMinima( xEref,yD,xE1,yE1,xE2,yE2);
    
    %punto F circulo rEF, linea BE
    %Para el cálculo de la posición faltante
    [ xF1,yF1,xF2,yF2 ] = lincir( xE,yE,xB,yB,xE,yE,EF);
    % Se escoge una de las dos soluciones
    [ xF,yF ] = distMinima( xFref,yFref,xF1,yF1,xF2,yF2);
    
    %punto G circulo rFG,c rGC
    %Para el cálculo de la posición faltante
    [ xG1,yG1,xG2,yG2 ] = circir( xF,yF,FG,xC,yC,GC);
    % Se escoge una de las dos soluciones
    [ xG,yG ] = distMinima( xGref,yF,xG1,yG1,xG2,yG2);
   
%IMPRIMIR RESULTADOS
fprintf('Posiciones Calculadas para phi=%0.2f\n',phi);
fprintf('A: %0.2fi + (%0.2fj)(m) \n',xA,yA);
fprintf('B: %0.2fi + (%0.2fj)(m) \n',xB,yB);
fprintf('C: %0.2fi + (%0.2fj)(m) \n',xC,yC);
fprintf('D: %0.2fi + (%0.2fj)(m) \n',xD,yD);
fprintf('E: %0.2fi + (%0.2fj)(m) \n',xE,yE);
fprintf('F: %0.2fi + (%0.2fj)(m) \n',xF,yF);
fprintf('G: %0.2fi + (%0.2fj)(m) \n',xG,yG);

%GRAFICAR EL MECANISMO
figure(1);
hold off
plot([xA,xB,xC],[yA,yB,yC],'r-o');
hold on;
plot([xF,xB,xE],[yF,yB,yE],'g-o');
plot([xD,xE],[yD,yE],'b-o');
plot([xC,xG],[yC,yG],'c-o');
text(xA,yA,' A'); text(xB,yB,' B'); text(xC,yC,' C');
text(xD,yD,' D'); text(xE,yE,' E'); text(xF,yF,' F');
text(xG,yG,' G');

title('Gráfica de posiciones')
xlabel('x (m)')
ylabel('y (m)')
axis([-0.8 2 -0.8 2]);
grid

