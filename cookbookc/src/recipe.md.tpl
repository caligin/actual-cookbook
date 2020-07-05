# {{recipe.title}}

{{recipe.description}}
## preparation

{{recipe.preparation}}
## ingredients

{{#each recipe.ingredients}}
- {{this.amount}}{{this.unit}} {{this.name}}{{/each}}

## notes

{{recipe.notes}}