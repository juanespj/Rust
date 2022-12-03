% Progarama para calcular el punto de intersecci�n de dos l�neas
% DATOS DE INGRESO
% Puntos de la primera l�nea (Ax,Ay) y (Bx,By)
% Puntos del dentro del circulo (Cx,Cy) y Radio R
% SALIDA
% Devuelve los puntos de intersecci�n (Px,Py) y (Qx,Qy)
function [ Px,Py,Qx,Qy ] = lincir( Ax,Ay,Bx,By,Cx,Cy,R)
    % Verificar si la recta es vertical o no
    if(Bx-Ax==0) % La recta es vertical
        %verificar que exista intersecci�n
        tmp=R^2-(Ax-Cx)^2;
        if(tmp<0)
            error('No existe intersecci�n entre l�nea y c�rculo');
        end
        Px=Ax;
        Qx=Ax;
        Py=Cy + tmp^0.5;
        Qy=Cy - tmp^0.5;
    else % La recta no es vertical
        m=(By-Ay)/(Bx-Ax);
        b=Ay-m*Ax;
        %verificar que exista intersecci�n
        tmp=- Cx^2*m^2 + 2*Cx*Cy*m - 2*Cx*b*m - Cy^2 + 2*Cy*b + R^2*m^2 + R^2 - b^2;
        if(tmp<0)
            error('No existe intersecci�n entre l�nea y c�rculo');
        end
        Px=(Cx - (tmp)^(1/2) + Cy*m - b*m)/(m^2 + 1);
        Qx=(Cx + (tmp)^(1/2) + Cy*m - b*m)/(m^2 + 1);
        Py=m*Px+b;
        Qy=m*Py+b;
     end
 end
