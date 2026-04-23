Param()
Write-Host "Starting test services..."
$compose = "tests/docker-compose.test.yml"
docker-compose -f $compose up -d --build

Write-Host "Waiting for Postgres to be ready..."
$retry = 0
while ($true) {
    $container = docker ps --filter "ancestor=postgres:15" --format "{{.ID}}"
    if ($container) {
        $res = docker exec $container pg_isready -U postgres -d faultreport_test 2>$null
        if ($LASTEXITCODE -eq 0) { break }
    }
    Start-Sleep -Seconds 1
    $retry++
    if ($retry -gt 60) { Write-Error "Postgres did not become ready in time."; exit 1 }
}
Write-Host "TEST_DATABASE_URL=postgresql://postgres:password@localhost:5433/faultreport_test"
Write-Host "Run tests with: TEST_DATABASE_URL=postgresql://postgres:password@localhost:5433/faultreport_test cargo test -- --ignored"
