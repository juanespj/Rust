%CALCULO DE LAS VELOCIDADES EN CADA POSICIÓN
clear all 
clc 
close all 
%DATOS INICIALES
AB=0.5; BC=0.6; %%barra2
DE=0.6;%barra4
yAD=-0.5;xAD=0.2;%1
BE=0.5;EF=1.2; %barra3
FG=0.6;GC=0.5;%barra5
phi=pi/16;
omega2=[0, 0, 2];
%CÁLCULO DE LAS POSICIONES
xA=0; 
yA=0;
y=0;
xD=xA+xAD; 
yD=yA+yAD;
%Puntos de referencia iniciales
xEref=DE;
yEref=yD;
yFref=BE;
xFref=xAD;
xGref=AB;
yGref=yA+FG;
%Calculo Posiciones con datos conocidos
xB=AB*cos(phi);
yB=AB*sin(phi);
xC=(AB+BC)*cos(phi);
yC=(AB+BC)*sin(phi);
%Se define el paso de la simulación
Paso=pi/180; 
J=1;
%variable simbólicas para el eslabón 2
omega3z = sym('omega3z','real');
omega4z = sym('omega4z','real');
omega5z = sym('omega5z','real');
omega6z = sym('omega6z','real');
 
for I=phi:Paso:phi+pi/4
    %Almacenar los ángulos
    ang(J)=I;
    %Calculo Posiciones con datos conocidos
    xB=AB*cos(I);
    yB=AB*sin(I);
    xC=(AB+BC)*cos(I);
    yC=(AB+BC)*sin(I);
    %punto E circulo rDE,c rBE
    %Para el cálculo de la posición faltante
    [ xE1,yE1,xE2,yE2 ] = circir( xB,yB,BE,xD,yD,BE);
    % Se escoge una de las dos soluciones
    [ xE,yE ] = distMinima( xEref,yEref,xE1,yE1,xE2,yE2);
    
    %punto F circulo rEF, linea BE
    %Para el cálculo de la posición faltante
    [ xF1,yF1,xF2,yF2 ] = lincir( xE,yE,xB,yB,xE,yE,EF);
    % Se escoge una de las dos soluciones
    [ xF,yF ] = distMinima( xFref,yFref,xF1,yF1,xF2,yF2);
    
    %punto G circulo rFG,c rGC
    %Para el cálculo de la posición faltante
    [ xG1,yG1,xG2,yG2 ] = circir( xF,yF,FG,xC,yC,GC);
    % Se escoge una de las dos soluciones
    [ xG,yG ] = distMinima( xGref,yGref,xG1,yG1,xG2,yG2);
    %Actualizacion de Referencias
    xEref=xE;
    yFref=yF;
    xFref=xF;
    xGref=xG;
    yGref=yG;
   
    %Contrucción de los vectores posición
    rA=[xA, yA, 0];
    rB=[xB, yB, 0];
    rC=[xC, yC, 0];
    rD=[xD, yD, 0];
    rE=[xE, yE, 0];
    rF=[xF, yF, 0];
    rG=[xG, yG, 0];
    vA=[0, 0, 0 ]; %en m/s
    %Barra 1
    %B y C Estan en la misma barra
    %Cálculo de la velocidad en B
    vB(J,:) = vA + cross(omega2,rB);
    vBmod(J)=norm(vB(J,:));
    %cálculo de la velocidad en C
    vC(J,:) = vA + cross(omega2,rC);
    vCmod(J)=norm(vC(J,:));
    %Barra 2
    vD=[0, 0, 0 ]; %en m/s
    %Barra 2 y Barra 3
    %F, B y E Estan en la misma barra     
    %Para esto es necesario crear dos variable simbólicas cuya declaración
    %se pone fuera del for porque se hace una sola vez
    Omega3 = [ 0 0 omega3z ];
    Omega4 = [ 0 0 omega4z ];    
    %Se usa las ecuaciónes vE = vD + w4×(rE -rD) 
    %y vE = vB + w3×(rE -rB) que en Matlab es
    eqvE=-( vD+cross(Omega4,rE- rD)) +  vB(J,:) + cross(Omega3,rE-rB);
    %como la ecuación anterior es vectorial la convertimos en dos
    %algebraicas
    eqvEx = eqvE(1); % Ecuación en X
    eqvEy = eqvE(2); % Ecuación en Y
    solvE = solve(eqvEx,eqvEy);
    omega3zs = eval(solvE.omega3z);
    omega4zs = eval(solvE.omega4z);
    omega3(J,:) = [0, 0, omega3zs];
    omega3mod(J)=norm(omega3(J,:));
    omega4(J,:) = [0, 0, omega4zs];
    omega4mod(J)=norm(omega4(J,:));
    vE(J,:) = vD + cross(omega4(J,:),rE-rD);
    vEmod(J)=norm(vE(J,:));
    vF(J,:) = vE(J,:) + cross(omega3(J,:),rF-rE);
    vFmod(J)=norm(vF(J,:));
    
    %Barra 4 y Barra 5
    %Fy G barra 5, C y G barra 4  
    Omega5 = [ 0 0 omega5z ];
    Omega6 = [ 0 0 omega6z ];    
    %Se usa las ecuaciónes vG = vF + w6×(rG -rF) 
    %y vG = vC + w5×(rG -rF) que en Matlab es
    eqvG=-( vF(J,:)+cross(Omega6,rG- rF)) +  vC(J,:) + cross(Omega5,rC-rF);
    %como la ecuación anterior es vectorial la convertimos en dos
    %algebraicas
    eqvGx = eqvG(1); % Ecuación en X
    eqvGy = eqvG(2); % Ecuación en Y
    solvG = solve(eqvGx,eqvGy);
    omega5zs = eval(solvG.omega5z);
    omega6zs = eval(solvG.omega6z);
    omega5(J,:) = [0, 0, omega5zs];
    omega5mod(J)=norm(omega5(J,:));
    omega6(J,:) = [0, 0, omega6zs];
    omega6mod(J)=norm(omega6(J,:));
    vG(J,:) = vF(J,:)+cross(omega6(J,:),rG- rF);
    vGmod(J)=norm(vG(J,:));    
    
    J=J+1;   
end
    %%BARRA 1
    figure(1);
    title('Velocidades Barra 1') 
    subplot(2,1,1);
    plot(ang*180/pi,vBmod);
    title('Modulo de la velocidad en B'...
        ,'Color','b','FontWeight','bold')  
    ylabel('Velocidad (m/s)')
    grid   
    subplot(2,1,2);
    plot(ang*180/pi,vCmod);
    title('Modulo de la velocidad en C'...
        ,'Color','b','FontWeight','bold')
    xlabel('Angulo \phi (Grados)')
    ylabel('Velocidad (m/s)')
    grid
    %%BARRA 2
    figure(2);
    title('Velocidades Barra 2')
    subplot(3,1,1);
    plot(ang*180/pi,vEmod);
    title('Modulo de la velocidad en E'...
        ,'Color','b','FontWeight','bold')
    grid
    subplot(3,1,2);
    plot(ang*180/pi,vBmod);
    title('Modulo de la velocidad en B'...
        ,'Color','b','FontWeight','bold')
    ylabel('Velocidad (m/s)')
    grid  
    subplot(3,1,3);
    plot(ang*180/pi,vFmod);
    title('Modulo de la velocidad en F'...
        ,'Color','b','FontWeight','bold')
    xlabel('Angulo \phi (Grados)')    
    grid
    
    %%BARRA 5 GC
    figure(3);
    title('Velocidades Barra 5')
    subplot(2,1,1);
    plot(ang*180/pi,vGmod);
    title('Modulo de la velocidad en G'...
        ,'Color','b','FontWeight','bold')    
    ylabel('Velocidad (m/s)')
    grid
    subplot(2,1,2);
    plot(ang*180/pi,vCmod);
    title('Modulo de la velocidad en C'...
        ,'Color','b','FontWeight','bold')
    xlabel('Angulo \phi (Grados)')
    ylabel('Velocidad (m/s)')
    grid
    
    %Graficas de velocidades angulares
    figure(4);
    title('Velocidades angulares')
    subplot(4,1,1);
    plot(ang*180/pi,omega3mod);
    title('Modulo de la velocidad angular en la barra3'...
        ,'Color','b','FontWeight','bold')
    
    grid
    subplot(4,1,2);
    plot(ang*180/pi,omega4mod);
    title('Modulo de la velocidad angular en la barra4'...
        ,'Color','b','FontWeight','bold') 
    ylabel('Velocidad angular (rad/s)')
    grid
    subplot(4,1,3);
    plot(ang*180/pi,omega5mod);
    title('Modulo de la velocidad angular en la barra5'...
        ,'Color','b','FontWeight','bold')    
    grid
    subplot(4,1,4);
    plot(ang*180/pi,omega6mod);
    title('Modulo de la velocidad angular en la barra6'...
        ,'Color','b','FontWeight','bold')    
    xlabel('Angulo \phi (Grados)')    
    grid
