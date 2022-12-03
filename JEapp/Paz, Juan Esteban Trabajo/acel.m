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
alpha2 = [0 0 -1 ]; % (rad/sˆ2)
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

%variable simbólicas para el eslabón 2: Aceleración
alpha3z=sym('alpha3z','real');
alpha4z=sym('alpha4z','real');
alpha5z=sym('alpha5z','real');
alpha6z=sym('alpha6z','real');

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
    %Para esto es necesario crear dos variable simbólicas 
    %cuya declaración se pone fuera del for porque
    %se hace una sola vez
    Omega3 = [ 0 0 omega3z ];
    Omega4 = [ 0 0 omega4z ];    
    %Se usa las ecuaciónes vE = vD + w4×(rE -rD)
    %y vE = vB + w3×(rE -rB) que en Matlab es
    eqvE=-( vD+cross(Omega4,rE- rD))...
        +  vB(J,:) + cross(Omega3,rE-rB);
    %como la ecuación anterior es vectorial 
    %la convertimos en dos algebraicas
    eqvEx = eqvE(1); % Ecuación en X
    eqvEy = eqvE(2); % Ecuación en Y
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
    %Se usa las ecuaciónes vG = vF + w6×(rG -rF) 
    %y vG = vC + w5×(rG -rF) que en Matlab es
    eqvG=-( vF(J,:)+cross(Omega6,rG- rF))...
        +  vC(J,:) + cross(Omega5,rC-rF);
    %como la ecuación anterior es vectorial 
    %la convertimos en dos algebraicas
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
            
    %Cálculo de la aceleración en B
    aA = [0 0 0 ];
    aD = [0 0 0 ];
    aB(J,:) = aA + cross(alpha2,rB-rA)...
        - dot(omega2,omega2)*(rB-rA);    
    aBmod(J)=norm(aB(J,:)); 
    %Cálculo de la aceleración en C
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
    eqaEx = eqaE(1); % Ecuación en X
    eqaEy = eqaE(2); % Ecuación en Y 
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
    eqaGx = eqaG(1); % Ecuación en X
    eqaGy = eqaG(2); % Ecuación en Y  
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
    title('Módulo de la aceleración en B'...
        ,'Color','b','FontWeight','bold')
    xlabel('Angulo \phi (Grados)')
    ylabel('Aceleración (m/s2)')
    grid
    subplot(2,1,2);
    plot(ang*180/pi,aCmod);
    title('Módulo de la aceleración en C'...
        ,'Color','b','FontWeight','bold')
    xlabel('Angulo \phi (Grados)')
    ylabel('Aceleración (m/s2)')
    grid
    %%BARRA 2 EBF
    figure(2);
    title('Aceleraciones en la Barra  2')
    subplot(3,1,1);
    plot(ang*180/pi,aEmod);
    title('Módulo de la aceleración en E'...
        ,'Color','b','FontWeight','bold')
    grid
    subplot(3,1,2);
    plot(ang*180/pi,aBmod);
    title('Módulo de la aceleración en B'...
        ,'Color','b','FontWeight','bold')
    ylabel('Velocidad (m/s)')
    grid  
    subplot(3,1,3);
    plot(ang*180/pi,aFmod);
    title('Módulo de la aceleración en F'...
        ,'Color','b','FontWeight','bold')
    xlabel('Angulo \phi (Grados)')    
    grid
    
    %%BARRA 5 GC
    figure(3);
    title('Velocidades Barra 5')
    subplot(2,1,1);
    plot(ang*180/pi,aGmod);
    title('Módulo de la aceleración en G'...
        ,'Color','b','FontWeight','bold')    
    ylabel('Velocidad (m/s)')
    grid
    subplot(2,1,2);
    plot(ang*180/pi,aCmod);
    title('Módulo de la aceleración en C'...
        ,'Color','b','FontWeight','bold')
    xlabel('Angulo \phi (Grados)')
    ylabel('Velocidad (m/s)')
    grid
    
    %Graficas de Aceleraciónes angulares
    figure(4);
    title('Aceleraciónes angulares')
    subplot(4,1,1);
    plot(ang*180/pi,alpha3mod);
    title('Modulo de la aceleración angular en la barra 2'...
    ,'Color','b','FontWeight','bold')
    grid
    subplot(4,1,2);
    plot(ang*180/pi,alpha4mod);
    title('Modulo de la aceleración angular en la barra 3'...
        ,'Color','b','FontWeight','bold') 
    ylabel('Velocidad angular (rad/s)')
    grid
    subplot(4,1,3);
    plot(ang*180/pi,alpha5mod);
    title('Modulo de la aceleración angular en la barra 4'...
        ,'Color','b','FontWeight','bold')    
    grid
    subplot(4,1,4);
    plot(ang*180/pi,alpha6mod);
    title('Modulo de la aceleración angular en la barra 5'...
        ,'Color','b','FontWeight','bold')    
    xlabel('Angulo \phi (Grados)')    
    grid
