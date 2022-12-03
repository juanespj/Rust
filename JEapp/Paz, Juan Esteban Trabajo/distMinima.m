% Progarama para calcular la distancia mínima entre dos puntos
% DATOS DE INGRESO
% Puntos de refererencia (Px,Py)
% Puntos a evaluar (Qx,Qy) y (Rx,Ry)
% SALIDA
% Devuelve el punto (Qx,Qy) o (Rx,Ry) mas cercano
function [ Sx,Sy ] = distMinima( Px,Py,Qx,Qy,Rx,Ry )
    %Cálculo de las distancias
    dist1=((Px-Qx)^2+(Py-Qy)^2)^2;
    dist2=((Px-Rx)^2+(Py-Ry)^2)^2;
    if(dist1<dist2)
        Sx=Qx;
        Sy=Qy;
    else
        Sx=Rx;
        Sy=Ry;
    end
end
