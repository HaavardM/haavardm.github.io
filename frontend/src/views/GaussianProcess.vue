<template>
  <div class="container-fluid">
    <GaussianProcess />

    <div class="container card my-4 text-box" style="width: 60rem;">
      <div class="card-body">
        <h1>Bayesian Methods: The Gaussian Process</h1>

        <p>
          When using mathematical models in my university courses I always
          struggled to accept the uncertainty of it all. In many of the courses,
          the models were either provided or we solved problems specifically
          designed to be easy to model. In was especially apparent in one of the
          control system courses where we spent a great deal of time
          investigating stability properties and robustness, without ever
          questioning the model itself.
        </p>
        <p>
          It wasn't until a course in linear systems theory we were introduced
          to the Kalman filter, and my concerns were finally taken into
          considerations. By combining linear systems theory with statistics,
          the Kalman filter attempted to quantify the uncertainty and adapt to
          how well the model performed. I did not know it then, but my eyes were
          opened to the world of Bayesian statistics.
        </p>
        <p>
          We will undoubtedly talk more about the Kalman filter later, but first
          I want to talk about a very different beast - the Gaussian Process. I
          may be an odd topic to start with as the theory might be hard to
          grasp, nevertheless, I find it as an intuitive and visual way to
          explore the mindset behind Bayesian methods.We will start simple and
          explore the behaviour of the Gaussian process before we gradually dive
          deeper into the theory. I assume you have some basic understanding of
          statistics, but as I believe there exists enough theoretical
          literature out there I hope to avoid it as much as I can.
        </p>

        <p>
          First of all, I hope you play around a bit with the example above. For
          those interested, it is implemented WebAssembly using Rust and the
          <a href="https://github.com/AtheMathmo/rusty-machine"
            >rusty-machine</a
          >
          crate, though there are many other implementations available. I
          especially would like you to notice how the shape of the red line and
          blue area change as you gradually remove the grey points. The red line
          corresponds to the most likely fit of the data, similar to what we
          would get from the simpler regression models. The blue area, however,
          is far more interesting and describes how confident the GP is about
          the fit. When the blue area is narrow, the model is confident in the
          predictions, and as the area grows the models gets increasingly
          uncertain. The area corresponds to the 95% confidence interval, and
          can formally be interpreted as the area we can be 95% certain the true
          value should be within.
        </p>
      </div>
    </div>
  </div>
</template>

<script>
// @ is an alias to /src
import GaussianProcess from "@/components/GaussianProcess.vue";

export default {
  name: "GaussianProcessView",
  components: {
    GaussianProcess,
  },
};
</script>

<style scoped lang="scss">
.text-box {
  h1 {
    margin: 2rem;
  }
  p {
    text-align: left;
  }
}
</style>
