export const y = [...Array(100).keys()].map(
  (x) => 20 + 4 * Math.sin((Math.PI * x) / 24) + 0.5 * Math.random()
);
export const x = [...y.keys()].map((i) => -1 * i).reverse();
