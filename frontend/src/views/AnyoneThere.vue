<template>
  <div class="container-fluid">
    <div class="container p-0">
      <AnyoneThere />
    </div>
    <div class="container card rounded my-4 shadow">
      <div class="py-4 px-2 text-left">
        <h1 class="text-center">What is this?</h1>
        <p>
          What your're looking at is a small embedded / cloud project I did for
          fun. The emoji indicates whether there is movement in front of the
          computer, i.e. if I am present in the room or not. The purpose of this
          project was mainly to play around with embedded electronics and cloud
          software.
        </p>
        <label>The project consist of three parts:</label>
        <ol>
          <li>An embedded Particle Core microcontroller with WiFi support.</li>
          <li>
            A backend for IoT event handling, using Google Cloud IoT Hub,
            Pub/Sub, Rust and GKE
          </li>
          <li>The frontend webapplication you are looking at.</li>
        </ol>
        <hr />
        <h2>Part 1: Embedded</h2>
        <span>
          The embedded part of the project is based on a Particle Core Dev Kit.
          The microcontroller is using a cheap PIR sensor to detect movement.
          When the state changes, the microcontroller sends a message using MQTT
          with TLS to Google Cloud IoT Hub (more on this later). In order to
          perform authentication with IoT Hub, a valid JWT needs to be signed by
          the device and used as password when connecting to the MQTT broker. As
          no JWT library was found among the public libraries for the Particle
          Core, the JWT was manually created and then signed using the
          opensource library mbedtls. The ES256 signature was generated using a
          sha256 hash and ECDSA with the secp256r1 (P-265) curve. Generating a
          valid JWT turned out to be more challenging than expected. Even after
          I was able to confirm the signature using OpenSSL, the JWT was still
          not accepted. In the end it turned out that the function
          <code>mbedtls_ecdsa_write_signature</code> automatically removes any
          leading zeros from the signature (which is common practice in many
          usecases), while JWT expects leading zeros to be included. Manually
          creating the ECDSA signature <code>r</code> and <code>s</code> solved
          the issue. The microcontroller is now able to automatically generate a
          valid JWT from a private key, detect movement in the apartment and
          send messages using MQTT.
        </span>
        <hr />
        <h2>Part 2: The Cloud</h2>
        <span>
          Google Cloud IoT Hub receives messages from the MQTT broker and
          automatically publish the messages on a predefined Pub/Sub topic. In
          order to display the state on this page, a Rust application was
          developed to consume messages from a Pub/Sub subscription, aggregate
          the values and respond to HTTP request with the current state. This
          was my second time working with Rust and first time using Rust in an
          application for use in Linux / K8S. A few problems turned out to be
          annoying:
          <ul>
            <li>
              <strike
                >Dependencies were a bit challenging. Several libraries imported
                <code>tokio</code>, but using different versions. This resulted
                in tokio complaining about multiple runtimes used in the same
                application. In the end I had to downgrade one of the libraries
                in order to avoid conflicts, forcing me to use an older release
                of tokio.</strike
              >
              No longer an issue as all libraries now use tokio 1.x.x.
            </li>
            <li>
              <strike
                >When deploying Go applications, I usually like to use alpine
                Docker images to reduce the image size. This appeared to work
                for Rust as well according to the docs, but I was unable to get
                it to build. Rust was not able to compile the application due to
                some missing libraries and even after installing
                <code>libc6-compat</code> it was still not working. I had to
                give up alpine for now, and reverted back to using a debian base
                image.</strike
              >
              Alpine Docker images is now working as expected! No clue whether
              it was the libraries or Rust itself, but I am happy!
            </li>
            <li>
              The Rust development experience is still not as good as I would've
              hoped. The VSCode plugin was not able to provide any autocomplete
              most of the time, even if it was seemingly able to correctly infer
              return types. I might be spoiled coming from Golang as my daily
              driver, but I expect basic autocompletion when working with such a
              strongly typed language.
            </li>
            <li>
              The build times are slow. This is annoying, but I guess it is
              worth as you get zero-cost abstractions. Comparing the compile
              times to a language such as Golang would not be fair.
            </li>
          </ul>
        </span>
        <span>
          Rust looks very promising, but to me it still feels immature. Most
          libraries are not yet considered stable, many libraries requires the
          nightly compiler, and the development tools (looking at you Rust
          VSCode plugin) still feels flaky. The generated Rust code is probably
          amazing, but I would still feel uncomfortable trusting the language
          for use in production systems.
        </span>
        <span>
          No persistent storage was used when implementing this application and
          the state will be lost if the K8S pod restarts. In the future I might
          (probably not) add Redis support or use a Google Cloud service such as
          Datastore.
        </span>
        <hr />
        <h2>Part 3: The UI</h2>
        The UI is simply an extension to this website, a place where I can put
        all my small projects which do not need an entire setup. This
        application was built as a custom component in Vue, and simply polls the
        Rust API frequenctly to retrieve the latest status. Some UI magic later,
        and you can see the results here.
      </div>
    </div>
  </div>
</template>

<script>
// @ is an alias to /src
import AnyoneThere from "@/components/AnyoneThere.vue";

export default {
  name: "AnyoneThereView",
  components: {
    AnyoneThere,
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

code {
  display: inline-block !important;
}
</style>
