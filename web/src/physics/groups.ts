/**
 * Grupos de colisão do Rapier: 16 bits de "sou" (alto) e 16 de "colido com"
 * (baixo). Cavalo e reboque não colidem entre si: a articulação é feita pela
 * junta da quinta-roda e as caixas simplificadas se sobreporiam em curvas
 * fechadas. Todo o resto usa o padrão (colide com tudo).
 */
const TRUCK = 0x0002;
const TRAILER = 0x0004;
const ALL = 0xffff;

const groups = (member: number, filter: number) => ((member << 16) | filter) >>> 0;

export const TRUCK_GROUPS = groups(TRUCK, ALL & ~TRAILER);
export const TRAILER_GROUPS = groups(TRAILER, ALL & ~TRUCK);
