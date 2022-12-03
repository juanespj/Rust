% Progarama para calcular el punto de intersección de dos líneas
% DATOS DE INGRESO
% Puntos de la primera línea (Ax,Ay) y (Bx,By)
% Puntos de la segunda línea (Cx,Cy) y (Dx,Dy)
% SALIDA
% Devuelve el punto de intersección (Px,Py)
function [ Px,Py ] = linlin( Ax,Ay,Bx,By,Cx,Cy,Dx,Dy)
    % calcular si la primera o segunda recta son verticales
    if(Bx-Ax==0) recVertA=1; else recVertA=0; end
    if(Dx-Cx==0) recVertB=1; else recVertB=0; end
    if(recVertA ==1 && recVertB==1)
        error('Las rectas son pareleas');
    end
    if(recVertA ==1 && recVertB==0) %primera recta vertical
        m2=((Dy-Cy)/(Dx-Cx));
        Px=Ax;
        Py=m2*(Px-Dx)+Dy;
    elseif(recVertA ==0 && recVertB==1)%segunda recta vertical
        m1=((By-Ay)/(Bx-Ax));
        Px=Cx;
        Py=m1*(Px-Bx)+By;
    else %Rectas en cualquier posición
        m1=((By-Ay)/(Bx-Ax));
        m2=((Dy-Cy)/(Dx-Cx));
        %Vericar nuevamente si las recatas no son paralelas
        if(m1 == m2)
            error('Las rectas son pareleas');
        end
        Px=(m1*Ax-m2*Dx+Dy-Ay)/(m1-m2);
        Py=m1*(Px-Ax)+ Ay;
    end
end
