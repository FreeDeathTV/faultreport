Param()

if (-not $env:TEST_DATABASE_URL) {
    Write-Error "TEST_DATABASE_URL must be set"
    exit 1
}

$migrationsDir = "backend/migrations"
if (-not (Test-Path $migrationsDir)) {
    Write-Error "migrations directory not found: $migrationsDir"
    exit 1
}

Write-Host "Applying migrations from $migrationsDir to $env:TEST_DATABASE_URL"

Get-ChildItem -Path $migrationsDir -Filter *.sql | Sort-Object Name | ForEach-Object {
    Write-Host "Applying $($_.FullName)"
    & psql $env:TEST_DATABASE_URL -f $_.FullName
}

Write-Host "Migrations applied."
