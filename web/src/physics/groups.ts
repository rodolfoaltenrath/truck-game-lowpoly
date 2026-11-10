/**
 * Grupos de colisão do Rapier: 16 bits de "sou" (alto) e 16 de "colido com"
 * (baixo). O chassi do cavalo não colide com o reboque (a quinta-roda fica
 * embaixo dele e a articulação é feita pela junta), mas a **cabine** colide:
 * num L fechado, a frente do reboque bate na traseira da cabine.
 * Todo o resto usa o padrão (colide com tudo).
 */
const TRUCK = 0x0002;
const TRAILER = 0x0004;
const CAB = 0x0008;
const ALL = 0xffff;

const groups = (member: number, filter: number) => ((member << 16) | filter) >>> 0;

export const TRUCK_GROUPS = groups(TRUCK, ALL & ~TRAILER);
export const CAB_GROUPS = groups(CAB, ALL);
export const TRAILER_GROUPS = groups(TRAILER, ALL & ~TRUCK);
