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
omega2=[0, 0, 2];% (rad/s)
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

%Calculo Masas,Inercias
lAC=AB+BC;
lDE=DE;
lFE=BE;
lGC=GC;
lFG=FG;

pesol=1;%Kg/m
m2=pesol*lAC;
m3=pesol*lFE;
m4=pesol*lDE;
m5=pesol*lGC;
m6=pesol*lFG;

Iac=1/12*m2*lAC^2;
IacA=Iac+m2*lAC^2/4;    
Ide=1/12*m4*lDE^2;
IdeD=Ide+m4*lDE^2/4;   
Ife=1/12*m3*lFE^2;    
Igc=1/12*m5*lGC^2;        
Ifg=1/12*m6*lFG^2;
%Se define el paso de la simulación
Paso=pi/180; 

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

%%Fuerzas Incognitas
FA = [sym('FAx','real') sym('FAy','real') 0 ];
F32 = [ sym('F32x','real') sym('F32y','real') 0 ];
F52 = [ sym('F52x','real') sym('F52y','real') 0 ];
F63 = [sym('F63x','real') sym('F63y','real') 0 ];
F43 = [sym('F43x','real') sym('F43y','real') 0 ];
F65 = [sym('F65x','real') sym('F65y','real') 0 ];
FD = [sym('FDx','real') sym('FDy','real') 0 ];      
I=phi;
J=1;

figure(1);
hold off
title('Gráfica de posiciones')
xlabel('x (m)')
ylabel('y (m)')
grid
axis([-0.8 2 -0.8 2]);
forcef=0.1;
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
    omega3s(J,:) = [0, 0, omega3zs];
    omega3mod(J)=norm(omega3s(J,:));
    omega4s(J,:) = [0, 0, omega4zs];
    omega3mod(J)=norm(omega4s(J,:));
    vE(J,:) = vD + cross(omega4s(J,:),rE-rD);
    vEmod(J)=norm(vE(J,:));
    vF(J,:) = vE(J,:) + cross(omega3s(J,:),rF-rE);
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
    omega5s(J,:) = [0, 0, omega5zs];
    omega5mod(J)=norm(omega5s(J,:));
    omega6s(J,:) = [0, 0, omega6zs];
    omega6mod(J)=norm(omega6s(J,:));
    vG(J,:) = vF(J,:)+cross(omega6s(J,:),rG- rF);
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
        -dot(omega3s(J,:),omega3s(J,:))*(rE-rB))...
    +aD+cross(Alpha4,rE-rD)...
        -dot(omega4s(J,:),omega4s(J,:))*(rE-rD);
    eqaEx = eqaE(1); % Ecuación en X
    eqaEy = eqaE(2); % Ecuación en Y 
    solaE = solve(eqaEx,eqaEy);
    alpha3zs=eval(solaE.alpha3z);
    alpha4zs=eval(solaE.alpha4z);
    alpha3s(J,:) = [0 0 alpha3zs];
    alpha3mod(J)=norm(alpha3s(J,:));
    alpha4s(J,:) = [0 0 alpha4zs];
    alpha4mod(J)=norm(alpha4s(J,:));
    aE(J,:)=aB(J,:)+cross(alpha3s(J,:),rE-rB)...
        -dot(omega3s(J,:),omega3s(J,:))*(rE-rB);  
    aEmod(J)=norm(aE(J,:));
    aF(J,:)=aB(J,:)+cross(alpha3s(J,:),rF-rB)...
        -dot(omega3s(J,:),omega3s(J,:))*(rF-rB);  
    aFmod(J)=norm(aF(J,:));
    
    %%Acel G
    Alpha5 = [ 0 0 alpha5z ]; % alpha5z unknown
    Alpha6 = [ 0 0 alpha6z ]; % alpha6z unknown
    
    eqaG=-(aF(J,:)+cross(Alpha6,rG-rF)...
        -dot(omega6s(J,:),omega6s(J,:))*(rG-rF))+...
    aC(J,:)+cross(Alpha5,rG-rC)...
        -dot(omega5s(J,:),omega5s(J,:))*(rG-rC);
    eqaGx = eqaG(1); % Ecuación en X
    eqaGy = eqaG(2); % Ecuación en Y  
    solaE = solve(eqaGx,eqaGy);
    alpha5zs=eval(solaE.alpha5z);
    alpha6zs=eval(solaE.alpha6z);
    alpha5s(J,:) = [0 0 alpha5zs];
    alpha5mod(J)=norm(alpha5s(J,:));
    alpha6s(J,:) = [0 0 alpha6zs];
    alpha6mod(J)=norm(alpha6s(J,:));
    aG(J,:)=aF(J,:)+cross(alpha6s(J,:),rG-rF)...
        -dot(omega6s(J,:),omega6s(J,:))*(rG-rF);  
    aGmod(J)=norm(aG(J,:));
   
   %%FUERZAS%%
    %BARRA 2 %FA,F32,F52,m2 a2cg
    a2cg(J,:)=cross(alpha2,(rF+rE)/2)...
        -dot(omega2,omega2)*(rF+rE)/2;   
    eqF2=FA+F32+F52-m2*a2cg(J,:);
    eqF2x = eqF2(1);
    eqF2y = eqF2(2);
    eqM2 = cross(rB-rA,F32)+cross(rC-rA,F52)-IacA*alpha2;
    eqM2z = eqM2(3);
    
    %BARRA 3 %F63,23,43,m3,a3cg
    a3cg(J,:)=aE(J,:)+cross(alpha3s(J,:),(rF+rE)/2)...
        -dot(omega3s(J,:),omega3s(J,:))*(rF+rE)/2;   
    F23 = -F32;    
    eqF3 = F23+F63+F43-m3*a3cg(J,:);
    eqF3x = eqF3(1);
    eqF3y = eqF3(2);
    eqM3 = cross(rB-(rE+rF)/2,F32)+cross(rF-(rE+rF)/2,F52)-Ife*alpha2;
    eqM3z = eqM3(3);
      
    %BARRA 5
    a5cg(J,:)=aC(J,:)+cross(alpha5s(J,:),(rG+rC)/2)...
        -dot(omega5s(J,:),omega5s(J,:))*(rG+rC)/2;   
    F25 = -F52;      
    eqF5 = F25+F65+-m5*a5cg(J,:);
    eqF5x = eqF5(1);
    eqF5y = eqF5(2);
    eqM5 = cross(rG-(rC+rG)/2,F65)-Igc*alpha5s(J,:);
    eqM5z = eqM5(3);
       
    %BARRA 6
    a6cg(J,:)=aF(J,:)+cross(alpha6s(J,:),(rG+rF)/2)...
        -dot(omega6s(J,:),omega6s(J,:))*(rG+rF)/2;  
    F56 = -F65;
    F36 = -F63;    
    eqF6 = F56+F36-m6*a6cg(J,:);
    eqF6x = eqF6(1);
    eqF6y = eqF6(2);
    eqM6 = cross(rG-(rF+rG)/2,F56)-Ifg*alpha6s(J,:);
    eqM6z = eqM6(3);
    %BARRA 4
    a4cg(J,:)=cross(alpha4s(J,:),(rE+rD)/2)...
        -dot(omega4s(J,:),omega4s(J,:))*(rE+rD)/2;      
    F34 = -F43;    
    eqF4 = FD+F34-m4*a4cg(J,:);
    eqF4x = eqF4(1);
    eqF4y = eqF4(2);
    
    
    %SOLUCION
    sol=solve(eqM3z,eqF3y,eqF3x,eqF2x,eqF2y,eqF3y,eqM2z,eqF4x,eqF4y,eqF6x,eqF6y,eqM6z,eqF5x,eqF5y,eqM5z);
    F32ys=eval(sol.F32y);
    F32xs=eval(sol.F32x);
    F52ys=eval(sol.F52y);
    F52xs=eval(sol.F52x);    
    F63ys=eval(sol.F63y);
    F63xs=eval(sol.F63x);
    F43xs=eval(sol.F43x);
    F43ys=eval(sol.F43y);   
    F65xs=eval(sol.F65x);
    F65ys=eval(sol.F65y);
    FAxs=eval(sol.FAx);
    FAys=eval(sol.FAy);
    FDxs=eval(sol.FDx);
    FDys=eval(sol.FDy); 
    
    F32s(J,:) = [ F32xs, F32ys, 0 ];
    F52s(J,:) = [ F52xs, F52ys, 0 ];
    F63s(J,:)  = [ F63xs, F63ys, 0 ];
    F43s(J,:)  = [ F43xs, F43ys, 0 ];
    F65s(J,:)  = [ F65xs, F65ys, 0 ];
    F23s(J,:)  = -F32s(J,:);
    F25s(J,:)  = -F52s(J,:);
    FAs(J,:) = [FAxs,FAys,0];
    FDs(J,:) = [FDxs,FDys,0];
    T4s = -cross(rE-rD,F32s(J,:))+IdeD*alpha4s(J,:);
    
    xAv(J)=xA;
    xBv(J)=xB;
    xCv(J)=xC;
    xDv(J)=xD;
    xEv(J)=xE;
    xFv(J)=xF;
    xGv(J)=xG;
    yAv(J)=yA;
    yBv(J)=yB;
    yCv(J)=yC;
    yDv(J)=yD;
    yEv(J)=yE;
    yFv(J)=yF;
    yGv(J)=yG;
    
  J=J+1;
end


    
for J=1:size(F23s,1)
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
    quiver3(xAv(J),yAv(J),0,FAs(J,1)*0.01,FAs(J,2)*0.01,0);%FA
    text(xAv(J)+FAs(J,1)*forcef,yAv(J)+FAs(J,2)*forcef,'FA');
    quiver3(xBv(J),yBv(J),0,F32s(J,1)*forcef,F32s(J,2)*forcef,0);%FB
    text(xBv(J)+F32s(J,1)*forcef,yBv(J)+F32s(J,2)*forcef,'FB'); 
    quiver3(xCv(J),yCv(J),0,F52s(J,1)*forcef,F52s(J,2)*forcef,0);%FC
    text(xCv(J)+F52s(J,1)*forcef,yCv(J)+F52s(J,2)*forcef,'FC');
    quiver3(xDv(J),yDv(J),0,FDs(J,1)*0.01,FDs(J,2)*0.01,0);%FD
    text(xDv(J)+FDs(J,1)*forcef,yDv(J)+FDs(J,2)*forcef,'FD');
    quiver3(xEv(J),yEv(J),0,F43s(J,1)*forcef,F43s(J,2)*forcef,0);%FE
    text(xEv(J)+F43s(J,1)*forcef,yEv(J)+F43s(J,2)*0.01,'FE');
    quiver3(xFv(J),yFv(J),0,F63s(J,1)*forcef,F63s(J,2)*forcef,0);%FF
    text(xFv(J)+F63s(J,1)*forcef,yFv(J)+F63s(J,2)*forcef,'F63');   
    quiver3(xGv(J),yGv(J),0,F65s(J,1)*0.01,F65s(J,2)*0.01,0);%FG
    text(xGv(J)+F65s(J,1)*forcef,yGv(J)+F65s(J,2)*forcef,'FG');
    hold off
    pause(0.1)
    J=J+1;
end

esfuerzoUltimo=400;%MPa
cortanteUltimo=esfuerzoUltimo/sqrt(3);
FS=2;
Tmax=max(T4s);
FBmax =norm(max(F32s));
FCmax =norm(max(F52s));
FFmax =norm(max(F63s));
FEmax =norm(max(F43s));
FGmax =norm(max(F65s));
FAmax=norm(max(FAs));
FDmax=norm(max(FDs));
dB=sqrt(FBmax*FS/(pi*cortantefluencia));
dC=sqrt(FCmax*FS/(pi*cortantefluencia));
dF=sqrt(FFmax*FS/(pi*cortantefluencia));
dE=sqrt(FEmax*FS/(pi*cortantefluencia));
dG=sqrt(FGmax*FS/(pi*cortantefluencia));
dA=sqrt(FAmax*FS/(pi*cortantefluencia));
dD=sqrt(FDmax*FS/(pi*cortantefluencia));
fprintf('Requerimiento de Torque:%0.3f\n',Tmax);
fprintf('diametro Pin A:%0.3f\n',dA);
fprintf('diametro Pin B:%0.3f\n',dB);
fprintf('diametro Pin C:%0.3f\n',dC);
fprintf('diametro Pin D:%0.3f\n',dD);
fprintf('diametro Pin E:%0.3f\n',dE);
fprintf('diametro Pin F:%0.3f\n',dF);
fprintf('diametro Pin G:%0.3f\n',dG);
