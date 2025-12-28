<!-- Improved compatibility of back to top link: See: https://github.com/othneildrew/Best-README-Template/pull/73 -->
<a id="readme-top"></a>
<!--
*** Thanks for checking out the Best-README-Template. If you have a suggestion
*** that would make this better, please fork the repo and create a pull request
*** or simply open an issue with the tag "enhancement".
*** Don't forget to give the project a star!
*** Thanks again! Now go create something AMAZING! :D
-->



<!-- PROJECT SHIELDS -->
<!--
*** I'm using markdown "reference style" links for readability.
*** Reference links are enclosed in brackets [ ] instead of parentheses ( ).
*** See the bottom of this document for the declaration of the reference variables
*** for contributors-url, forks-url, etc. This is an optional, concise syntax you may use.
*** https://www.markdownguide.org/basic-syntax/#reference-style-links
-->
<!-- [![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![project_license][license-shield]][license-url]
[![LinkedIn][linkedin-shield]][linkedin-url] -->

[![continuous integration](https://github.com/leungkkf/rs-iiif-browser/actions/workflows/rust.yml/badge.svg)](https://github.com/leungkkf/rs-iiif-browser/actions/workflows/rust.yml)


<!-- PROJECT LOGO -->
<br />
<div align="center">

<h3 align="center">Rust IIIF Browser</h3>

  <p align="center">
    A proof-of-concept IIIF browser built using Rust for viewing <a href="https://iiif.io">IIIF manifests and images</a>.
    <br />
    <br />
    <br />
    <a href="https://github.com/leungkkf/rs-iiif-browser/issues/new?labels=bug&template=bug-report---.md">Report Bug</a>
    <a href="https://github.com/leungkkf/rs-iiif-browser/issues/new?labels=enhancement&template=feature-request---.md">Request Feature</a>
  </p>
</div>



<!-- TABLE OF CONTENTS -->
<details>
  <summary>Table of Contents</summary>
  <ol>
    <li>
      <a href="#about-the-project">About The Project</a>
      <ul>
        <li><a href="#built-with">Built With</a> Rust</li>
      </ul>
    </li>
    <!-- <li>
      <a href="#getting-started">Getting Started</a>
      <ul>
        <li><a href="#prerequisites">Prerequisites</a></li>
        <li><a href="#installation">Installation</a></li>
      </ul>
    </li> -->
    <!-- <li><a href="#usage">Usage</a></li>
    <li><a href="#roadmap">Roadmap</a></li> -->
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
    <!-- <li><a href="#contact">Contact</a></li>
    <li><a href="#acknowledgments">Acknowledgments</a></li> -->
  </ol>
</details>



<!-- ABOUT THE PROJECT -->
## About The Project

The IIIF Browser provides a proof-of-concept client built using Rust for viewing IIIF manifests and images. 
It is based on the [Bevy](https://bevy.org/) game engine with the [Bevy egui plugin](https://github.com/vladbat00/bevy_egui) (which in turn uses [egui](https://github.com/emilk/egui)). 
Bevy allows easy rendering of the tiles/images with support for camera and input. 
The project is created as an exercise while learning Rust. 

A web demo can be found [here](https://leungkkf.github.io/testbed/). 


Existing features: 
* Limited support of IIIF manifest and image (versions 2 and 3).
* Pan and deep zoom.
* Minimap.
* Canvas thumbnails on the side panel.
* Cross-platform builds (tried on the following platforms)
  * Windows
  * Linux
  * Wasm (issues with touch controls and virtual keyboard)
  * Android (issues with touch controls and virtual keyboard)
* Limited support for 3D. 

<p align="right">(<a href="#readme-top">back to top</a>)</p>




<!-- GETTING STARTED -->
<!-- ## Getting Started

This is an example of how you may give instructions on setting up your project locally.
To get a local copy up and running follow these simple example steps.

### Prerequisites

This is an example of how to list things you need to use the software and how to install them.
* npm
  ```sh
  npm install npm@latest -g
  ```

### Installation

1. Get a free API Key at [https://example.com](https://example.com)
2. Clone the repo
   ```sh
   git clone https://github.com/github_username/repo_name.git
   ```
3. Install NPM packages
   ```sh
   npm install
   ```
4. Enter your API in `config.js`
   ```js
   const API_KEY = 'ENTER YOUR API';
   ```
5. Change git remote url to avoid accidental pushes to base project
   ```sh
   git remote set-url origin github_username/repo_name
   git remote -v # confirm the changes
   ```

<p align="right">(<a href="#readme-top">back to top</a>)</p> -->



<!-- USAGE EXAMPLES -->
<!-- ## Usage

Use this space to show useful examples of how a project can be used. Additional screenshots, code examples and demos work well in this space. You may also link to more resources.

_For more examples, please refer to the [Documentation](https://example.com)_

<p align="right">(<a href="#readme-top">back to top</a>)</p> -->



<!-- ROADMAP -->
## To-Do List

Some ideas are listed below:

- [ ] Better support for IIIF manifest and image.
- [ ] Better support for 3D.
- [ ] Support for IIIF collection and discovery.
- [ ] Support for virtual keyboard in mobile devices.
- [ ] ...
    

<p align="right">(<a href="#readme-top">back to top</a>)</p> 



<!-- CONTRIBUTING -->
## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create. Any contributions you make are **greatly appreciated**.

If you have a suggestion that would make this better, please fork the repo and create a pull request. You can also simply open an issue with the tag "enhancement".
Don't forget to give the project a star! Thanks again!

1. Fork the Project
2. Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3. Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the Branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

<p align="right">(<a href="#readme-top">back to top</a>)</p>


<!-- LICENSE -->
## License

Distributed under the GPL licence. See `LICENSE` for more information.

<p align="right">(<a href="#readme-top">back to top</a>)</p>



