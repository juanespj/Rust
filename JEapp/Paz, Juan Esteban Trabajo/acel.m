%CALCULO DE LAS VELOCIDADES EN CADA POSICI�N
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
alpha2 = [0 0 -1 ]; % (rad/s�2)
%C�LCULO DE LAS POSICIONES
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
%Se define el paso de la simulaci�n
Paso=pi/180; 
J=1;
%variable simb�licas para el eslab�n 2
omega3z = sym('omega3z','real');
omega4z = sym('omega4z','real');
omega5z = sym('omega5z','real');
omega6z = sym('omega6z','real');

%variable simb�licas para el eslab�n 2: Aceleraci�n
alpha3z=sym('alpha3z','real');
alpha4z=sym('alpha4z','real');
alpha5z=sym('alpha5z','real');
alpha6z=sym('alpha6z','real');

for I=phi:Paso:phi+pi/4
    %Almacenar los �ngulos
    ang(J)=I;
    %Calculo Posiciones con datos conocidos
    xB=AB*cos(I);
    yB=AB*sin(I);
    xC=(AB+BC)*cos(I);
    yC=(AB+BC)*sin(I);
    %punto E circulo rDE,c rBE
    %Para el c�lculo de la posici�n faltante
    [ xE1,yE1,xE2,yE2 ] = circir( xB,yB,BE,xD,yD,BE);
    % Se escoge una de las dos soluciones
    [ xE,yE ] = distMinima( xEref,yEref,xE1,yE1,xE2,yE2);
    
    %punto F circulo rEF, linea BE
    %Para el c�lculo de la posici�n faltante
    [ xF1,yF1,xF2,yF2 ] = lincir( xE,yE,xB,yB,xE,yE,EF);
    % Se escoge una de las dos soluciones
    [ xF,yF ] = distMinima( xFref,yFref,xF1,yF1,xF2,yF2);
    
    %punto G circulo rFG,c rGC
    %Para el c�lculo de la posici�n faltante
    [ xG1,yG1,xG2,yG2 ] = circir( xF,yF,FG,xC,yC,GC);
    % Se escoge una de las dos soluciones
    [ xG,yG ] = distMinima( xGref,yGref,xG1,yG1,xG2,yG2);
    %Actualizacion de Referencias
    xEref=xE;
    yFref=yF;
    xFref=xF;
    xGref=xG;
    yGref=yG;
   
    %Contrucci�n de los vectores posici�n
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
    %C�lculo de la velocidad en B
    vB(J,:) = vA + cross(omega2,rB);
    vBmod(J)=norm(vB(J,:));
    %c�lculo de la velocidad en C
    vC(J,:) = vA + cross(omega2,rC);
    vCmod(J)=norm(vC(J,:));
    %Barra 2
    vD=[0, 0, 0 ]; %en m/s
    %Barra 2 y Barra 3
    %F, B y E Estan en la misma barra     
    %Para esto es necesario crear dos variable simb�licas 
    %cuya declaraci�n se pone fuera del for porque
    %se hace una sola vez
    Omega3 = [ 0 0 omega3z ];
    Omega4 = [ 0 0 omega4z ];    
    %Se usa las ecuaci�nes vE = vD + w4�(rE -rD)
    %y vE = vB + w3�(rE -rB) que en Matlab es
    eqvE=-( vD+cross(Omega4,rE- rD))...
        +  vB(J,:) + cross(Omega3,rE-rB);
    %como la ecuaci�n anterior es vectorial 
    %la convertimos en dos algebraicas
    eqvEx = eqvE(1); % Ecuaci�n en X
    eqvEy = eqvE(2); % Ecuaci�n en Y
    solvE = solve(eqvEx,eqvEy);
    omega3zs = eval(solvE.omega3z);
    omega4zs = eval(solvE.omega4z);
    omega3(J,:) = [0, 0, omega3zs];
    omega3mod(J)=norm(omega3(J,:));
    omega4(J,:) = [0, 0, omega4zs];
    omega3mod(J)=norm(omega4(J,:));
    vE(J,:) = vD + cross(omega4(J,:),rE-rD);
    vEmod(J)=norm(vE(J,:));
    vF(J,:) = vE(J,:) + cross(omega3(J,:),rF-rE);
    vFmod(J)=norm(vF(J,:));
    
    %Barra 5 y Barra 6
    %Fy G barra 6, C y G barra 5  
    Omega5 = [ 0 0 omega5z ];
    Omega6 = [ 0 0 omega6z ];    
    %Se usa las ecuaci�nes vG = vF + w6�(rG -rF) 
    %y vG = vC + w5�(rG -rF) que en Matlab es
    eqvG=-( vF(J,:)+cross(Omega6,rG- rF))...
        +  vC(J,:) + cross(Omega5,rC-rF);
    %como la ecuaci�n anterior es vectorial 
    %la convertimos en dos algebraicas
    eqvGx = eqvG(1); % Ecuaci�n en X
    eqvGy = eqvG(2); % Ecuaci�n en Y
    solvG = solve(eqvGx,eqvGy);
    omega5zs = eval(solvG.omega5z);
    omega6zs = eval(solvG.omega6z);
    omega5(J,:) = [0, 0, omega5zs];
    omega5mod(J)=norm(omega5(J,:));
    omega6(J,:) = [0, 0, omega6zs];
    omega6mod(J)=norm(omega6(J,:));
    vG(J,:) = vF(J,:)+cross(omega6(J,:),rG- rF);
    vGmod(J)=norm(vG(J,:));  
            
    %C�lculo de la aceleraci�n en B
    aA = [0 0 0 ];
    aD = [0 0 0 ];
    aB(J,:) = aA + cross(alpha2,rB-rA)...
        - dot(omega2,omega2)*(rB-rA);    
    aBmod(J)=norm(aB(J,:)); 
    %C�lculo de la aceleraci�n en C
    aC(J,:) = aA + cross(alpha2,rC-rA)...
        - dot(omega2,omega2)*(rC-rA);
    aCmod(J)=norm(aC(J,:)); 
    %%Acel E
    Alpha3 = [ 0 0 alpha3z ]; % alpha3z unknown
    Alpha4 = [ 0 0 alpha4z ]; % alpha3z unknown
    
    eqaE=-(aB(J,:)+cross(Alpha3,rE-rB)...
        -dot(omega3(J,:),omega3(J,:))*(rE-rB))...
    +aD+cross(Alpha4,rE-rD)...
        -dot(omega4(J,:),omega4(J,:))*(rE-rD);
    eqaEx = eqaE(1); % Ecuaci�n en X
    eqaEy = eqaE(2); % Ecuaci�n en Y 
    solaE = solve(eqaEx,eqaEy);
    alpha3zs=eval(solaE.alpha3z);
    alpha4zs=eval(solaE.alpha4z);
    alpha3(J,:) = [0 0 alpha3zs];
    alpha3mod(J)=norm(alpha3(J,:));
    alpha4(J,:) = [0 0 alpha4zs];
    alpha4mod(J)=norm(alpha4(J,:));
    aE(J,:)=aB(J,:)+cross(alpha3(J,:),rE-rB)...
        -dot(omega3(J,:),omega3(J,:))*(rE-rB);  
    aEmod(J)=norm(aE(J,:));
    aF(J,:)=aB(J,:)+cross(alpha3(J,:),rF-rB)...
        -dot(omega3(J,:),omega3(J,:))*(rF-rB);  
    aFmod(J)=norm(aF(J,:));
    
    %%Acel G
    Alpha5 = [ 0 0 alpha5z ]; % alpha5z unknown
    Alpha6 = [ 0 0 alpha6z ]; % alpha6z unknown
    
    eqaG=-(aF(J,:)+cross(Alpha6,rG-rF)...
        -dot(omega6(J,:),omega6(J,:))*(rG-rF))+...
    aC(J,:)+cross(Alpha5,rG-rC)...
        -dot(omega5(J,:),omega5(J,:))*(rG-rC);
    eqaGx = eqaG(1); % Ecuaci�n en X
    eqaGy = eqaG(2); % Ecuaci�n en Y  
    solaE = solve(eqaGx,eqaGy);
    alpha5zs=eval(solaE.alpha5z);
    alpha6zs=eval(solaE.alpha6z);
    alpha5(J,:) = [0 0 alpha5zs];
    alpha5mod(J)=norm(alpha5(J,:));
    alpha6(J,:) = [0 0 alpha6zs];
    alpha6mod(J)=norm(alpha6(J,:));
    aG(J,:)=aF(J,:)+cross(alpha6(J,:),rG-rF)...
        -dot(omega6(J,:),omega6(J,:))*(rG-rF);  
    aGmod(J)=norm(aG(J,:));
    J=J+1;
end      
  
    %%BARRA 1 BC
    figure(1);
    title('Aceleraciones en la Barra 1') 
    subplot(2,1,1);
    plot(ang*180/pi,aBmod);
    title('M�dulo de la aceleraci�n en B'...
        ,'Color','b','FontWeight','bold')
    xlabel('Angulo \phi (Grados)')
    ylabel('Aceleraci�n (m/s2)')
    grid
    subplot(2,1,2);
    plot(ang*180/pi,aCmod);
    title('M�dulo de la aceleraci�n en C'...
        ,'Color','b','FontWeight','bold')
    xlabel('Angulo \phi (Grados)')
    ylabel('Aceleraci�n (m/s2)')
    grid
    %%BARRA 2 EBF
    figure(2);
    title('Aceleraciones en la Barra  2')
    subplot(3,1,1);
    plot(ang*180/pi,aEmod);
    title('M�dulo de la aceleraci�n en E'...
        ,'Color','b','FontWeight','bold')
    grid
    subplot(3,1,2);
    plot(ang*180/pi,aBmod);
    title('M�dulo de la aceleraci�n en B'...
        ,'Color','b','FontWeight','bold')
    ylabel('Velocidad (m/s)')
    grid  
    subplot(3,1,3);
    plot(ang*180/pi,aFmod);
    title('M�dulo de la aceleraci�n en F'...
        ,'Color','b','FontWeight','bold')
    xlabel('Angulo \phi (Grados)')    
    grid
    
    %%BARRA 5 GC
    figure(3);
    title('Velocidades Barra 5')
    subplot(2,1,1);
    plot(ang*180/pi,aGmod);
    title('M�dulo de la aceleraci�n en G'...
        ,'Color','b','FontWeight','bold')    
    ylabel('Velocidad (m/s)')
    grid
    subplot(2,1,2);
    plot(ang*180/pi,aCmod);
    title('M�dulo de la aceleraci�n en C'...
        ,'Color','b','FontWeight','bold')
    xlabel('Angulo \phi (Grados)')
    ylabel('Velocidad (m/s)')
    grid
    
    %Graficas de Aceleraci�nes angulares
    figure(4);
    title('Aceleraci�nes angulares')
    subplot(4,1,1);
    plot(ang*180/pi,alpha3mod);
    title('Modulo de la aceleraci�n angular en la barra 2'...
    ,'Color','b','FontWeight','bold')
    grid
    subplot(4,1,2);
    plot(ang*180/pi,alpha4mod);
    title('Modulo de la aceleraci�n angular en la barra 3'...
        ,'Color','b','FontWeight','bold') 
    ylabel('Velocidad angular (rad/s)')
    grid
    subplot(4,1,3);
    plot(ang*180/pi,alpha5mod);
    title('Modulo de la aceleraci�n angular en la barra 4'...
        ,'Color','b','FontWeight','bold')    
    grid
    subplot(4,1,4);
    plot(ang*180/pi,alpha6mod);
    title('Modulo de la aceleraci�n angular en la barra 5'...
        ,'Color','b','FontWeight','bold')    
    xlabel('Angulo \phi (Grados)')    
    grid
