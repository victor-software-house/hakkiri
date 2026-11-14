## ADDED Requirements

### Requirement: A package style named Vale is skipped with a warning

hakkiri SHALL skip a package folder named for the built-in style `Vale`, as Vale does, and SHALL print one warning that names the folder and says the built-in style wins.  
The other styles of the package SHALL load as before.

#### Scenario: A package holds styles Acme and Vale

- **WHEN** a package holds `styles/Acme` and `styles/Vale`, and a glob lists `styles: [Acme, Vale]`
- **THEN** the rules of `Acme` run, no rule of the folder `Vale` runs, and `hakkiri check` prints one warning that `styles/Vale` is skipped because the built-in style `Vale` wins
