export const y = [...Array(80).keys()].map(
  () =>
    Math.sqrt(-2 * Math.log(Math.random())) *
      Math.cos(2 * Math.PI * Math.random()) *
      2 +
    20
);
export const x = [...y.keys()].map((i) => -1 * i).reverse();
